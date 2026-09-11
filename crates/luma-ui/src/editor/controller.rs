// Text shaping and platform input handling follow GPUI's Apache-2.0
// `examples/input.rs` at Zed commit 251854020a9dbea1a388bf392c9bd04fd136a557.

use std::ops::Range;

use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, FocusHandle, Focusable, GlobalElementId, HighlightStyle, KeyBinding,
    LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point,
    Role, SharedString, StyledText, Subscription, TextLayout, UTF16Selection, UnderlineStyle,
    Window, actions, div, fill, hsla, point, prelude::*, px, rgb, size,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::text_offset::{
    byte_range_to_utf16, byte_to_utf16_offset, nearest_grapheme_boundary, next_grapheme_boundary,
    previous_grapheme_boundary, utf16_range_to_bytes, utf16_range_to_bytes_in,
};

use super::{
    document::{ByteOffset, Document, EditBatch},
    history::{EditHistory, EditKind, EditTransaction, HistorySnapshot},
    layout::{LayoutViewport, reveal_scroll},
    session::{CompositionState, EditorSession, Selection, TransactionToken},
};

const LINE_HEIGHT: Pixels = px(22.);
const EDITOR_PADDING: Pixels = px(8.);
const GUTTER_WIDTH: Pixels = px(52.);

struct Replacement<'a> {
    range: Range<usize>,
    text: &'a str,
    selection: Selection,
    kind: Option<EditKind>,
    before: Option<HistorySnapshot>,
}

actions!(
    luma_editor,
    [
        Backspace,
        Delete,
        Left,
        Right,
        Up,
        Down,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectAll,
        Home,
        End,
        Undo,
        Redo,
        Paste,
        Cut,
        Copy,
        InsertNewline,
    ]
);

pub(crate) fn register_key_bindings(context: &mut App) {
    let editor = Some("LumaEditor");
    context.bind_keys([
        KeyBinding::new("backspace", Backspace, editor),
        KeyBinding::new("delete", Delete, editor),
        KeyBinding::new("left", Left, editor),
        KeyBinding::new("right", Right, editor),
        KeyBinding::new("up", Up, editor),
        KeyBinding::new("down", Down, editor),
        KeyBinding::new("shift-left", SelectLeft, editor),
        KeyBinding::new("shift-right", SelectRight, editor),
        KeyBinding::new("shift-up", SelectUp, editor),
        KeyBinding::new("shift-down", SelectDown, editor),
        KeyBinding::new("home", Home, editor),
        KeyBinding::new("end", End, editor),
        KeyBinding::new("cmd-a", SelectAll, editor),
        KeyBinding::new("ctrl-a", SelectAll, editor),
        KeyBinding::new("cmd-z", Undo, editor),
        KeyBinding::new("ctrl-z", Undo, editor),
        KeyBinding::new("shift-cmd-z", Redo, editor),
        KeyBinding::new("ctrl-y", Redo, editor),
        KeyBinding::new("cmd-c", Copy, editor),
        KeyBinding::new("ctrl-c", Copy, editor),
        KeyBinding::new("cmd-x", Cut, editor),
        KeyBinding::new("ctrl-x", Cut, editor),
        KeyBinding::new("cmd-v", Paste, editor),
        KeyBinding::new("ctrl-v", Paste, editor),
        KeyBinding::new("enter", InsertNewline, editor),
    ]);
}

pub(crate) struct EditorController {
    focus_handle: FocusHandle,
    document: Document,
    session: EditorSession,
    history: EditHistory,
    viewport: LayoutViewport,
    last_layout: Option<TextLayout>,
    is_selecting: bool,
    saved_text: String,
    pending_edit_kind: Option<EditKind>,
    pending_history_before: Option<HistorySnapshot>,
    composition_before: Option<HistorySnapshot>,
    next_transaction_token: u64,
    _subscriptions: Vec<Subscription>,
}

impl EditorController {
    pub(crate) fn new(
        text: impl Into<String>,
        window: &mut Window,
        context: &mut Context<Self>,
    ) -> Self {
        let document = Document::new(text);
        let saved_text = document.text().to_owned();
        let revision = document.revision().get();
        let focus_handle = context.focus_handle().tab_stop(true);
        let focus_out = context.on_focus_out(&focus_handle, window, |editor, _, _, context| {
            editor.finish_composition(context);
            editor.history.barrier();
            editor.is_selecting = false;
        });
        Self {
            focus_handle,
            session: EditorSession::new(&document),
            document,
            history: EditHistory::new(),
            viewport: LayoutViewport::new(revision),
            last_layout: None,
            is_selecting: false,
            saved_text,
            pending_edit_kind: None,
            pending_history_before: None,
            composition_before: None,
            next_transaction_token: 1,
            _subscriptions: vec![focus_out],
        }
    }

    pub(crate) fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.document.text() != self.saved_text
    }

    pub(crate) fn cursor_position(&self) -> (usize, usize) {
        let offset = self.session.selection().active();
        let line = self.document.line_index(offset).unwrap_or_default();
        let line_start = self.document.line_start(line).unwrap_or_default().get();
        let column = self.document.text()[line_start..offset.get()]
            .graphemes(true)
            .count();
        (line + 1, column + 1)
    }

    fn cursor_offset(&self) -> usize {
        self.session.selection().active().get()
    }

    fn selection_range(&self) -> Range<usize> {
        let range = self.session.selection().range();
        range.start.get()..range.end.get()
    }

    fn snapshot(&self) -> HistorySnapshot {
        HistorySnapshot::new(self.document.text(), self.session.selection())
    }

    fn set_selection(&mut self, selection: Selection, context: &mut Context<Self>) {
        if self.session.composition().is_some() || self.composition_before.is_some() {
            self.finish_composition(context);
        }
        if self
            .session
            .set_selection(&self.document, selection)
            .is_err()
        {
            return;
        }
        self.session.clear_preferred_horizontal_x();
        self.history.barrier();
        self.viewport.request_caret_visible();
        context.notify();
    }

    fn move_to(&mut self, offset: usize, context: &mut Context<Self>) {
        let offset = nearest_grapheme_boundary(self.document.text(), offset);
        self.set_selection(Selection::caret(ByteOffset::new(offset)), context);
    }

    fn select_to(&mut self, offset: usize, context: &mut Context<Self>) {
        let offset = nearest_grapheme_boundary(self.document.text(), offset);
        let anchor = self.session.selection().anchor();
        self.set_selection(Selection::new(anchor, ByteOffset::new(offset)), context);
    }

    fn move_vertical(&mut self, delta: isize, selecting: bool, context: &mut Context<Self>) {
        let (line, column) = line_and_column(self.document.text(), self.cursor_offset());
        let preferred = self
            .session
            .preferred_horizontal_x()
            .map(|value| value as usize)
            .unwrap_or(column);
        let line_count = self.document.line_count();
        let target_line = line
            .saturating_add_signed(delta)
            .min(line_count.saturating_sub(1));
        let offset = offset_for_line_column(self.document.text(), target_line, preferred);
        let anchor = self.session.selection().anchor();
        if selecting {
            self.set_selection(Selection::new(anchor, ByteOffset::new(offset)), context);
        } else {
            self.set_selection(Selection::caret(ByteOffset::new(offset)), context);
        }
        self.session.set_preferred_horizontal_x(preferred as f32);
    }

    fn left(&mut self, _: &Left, _: &mut Window, context: &mut Context<Self>) {
        let selection = self.session.selection();
        if selection.is_collapsed() {
            self.move_to(
                previous_grapheme_boundary(self.document.text(), self.cursor_offset()),
                context,
            );
        } else {
            self.move_to(selection.range().start.get(), context);
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, context: &mut Context<Self>) {
        let selection = self.session.selection();
        if selection.is_collapsed() {
            self.move_to(
                next_grapheme_boundary(self.document.text(), self.cursor_offset()),
                context,
            );
        } else {
            self.move_to(selection.range().end.get(), context);
        }
    }

    fn up(&mut self, _: &Up, _: &mut Window, context: &mut Context<Self>) {
        self.move_vertical(-1, false, context);
    }

    fn down(&mut self, _: &Down, _: &mut Window, context: &mut Context<Self>) {
        self.move_vertical(1, false, context);
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, context: &mut Context<Self>) {
        self.select_to(
            previous_grapheme_boundary(self.document.text(), self.cursor_offset()),
            context,
        );
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, context: &mut Context<Self>) {
        self.select_to(
            next_grapheme_boundary(self.document.text(), self.cursor_offset()),
            context,
        );
    }

    fn select_up(&mut self, _: &SelectUp, _: &mut Window, context: &mut Context<Self>) {
        self.move_vertical(-1, true, context);
    }

    fn select_down(&mut self, _: &SelectDown, _: &mut Window, context: &mut Context<Self>) {
        self.move_vertical(1, true, context);
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, context: &mut Context<Self>) {
        self.set_selection(
            Selection::new(ByteOffset::new(0), self.document.len()),
            context,
        );
    }

    fn home(&mut self, _: &Home, _: &mut Window, context: &mut Context<Self>) {
        let (line, _) = line_and_column(self.document.text(), self.cursor_offset());
        let offset = self
            .document
            .line_start(line)
            .map(ByteOffset::get)
            .unwrap_or_default();
        self.move_to(offset, context);
    }

    fn end(&mut self, _: &End, _: &mut Window, context: &mut Context<Self>) {
        let (line, _) = line_and_column(self.document.text(), self.cursor_offset());
        self.move_to(line_content_end(self.document.text(), line), context);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, context: &mut Context<Self>) {
        self.pending_history_before = Some(self.snapshot());
        if self.session.selection().is_collapsed() {
            let previous = previous_grapheme_boundary(self.document.text(), self.cursor_offset());
            if previous == self.cursor_offset() {
                self.pending_history_before = None;
                window.play_system_bell();
                return;
            }
            let active = self.session.selection().active();
            if self
                .session
                .set_selection(
                    &self.document,
                    Selection::new(active, ByteOffset::new(previous)),
                )
                .is_err()
            {
                self.pending_history_before = None;
                window.play_system_bell();
                return;
            }
        }
        self.pending_edit_kind = Some(EditKind::Typing);
        self.replace_text_in_range(None, "", window, context);
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, context: &mut Context<Self>) {
        self.pending_history_before = Some(self.snapshot());
        if self.session.selection().is_collapsed() {
            let next = next_grapheme_boundary(self.document.text(), self.cursor_offset());
            if next == self.cursor_offset() {
                self.pending_history_before = None;
                window.play_system_bell();
                return;
            }
            let active = self.session.selection().active();
            if self
                .session
                .set_selection(
                    &self.document,
                    Selection::new(active, ByteOffset::new(next)),
                )
                .is_err()
            {
                self.pending_history_before = None;
                window.play_system_bell();
                return;
            }
        }
        self.pending_edit_kind = Some(EditKind::Barrier);
        self.replace_text_in_range(None, "", window, context);
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, context: &mut Context<Self>) {
        if let Some(text) = context.read_from_clipboard().and_then(|item| item.text()) {
            self.pending_edit_kind = Some(EditKind::Barrier);
            self.replace_text_in_range(None, &text, window, context);
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, context: &mut Context<Self>) {
        if let Some(text) = self.document.text().get(self.selection_range())
            && !text.is_empty()
        {
            context.write_to_clipboard(ClipboardItem::new_string(text.to_owned()));
        }
    }

    fn cut(&mut self, _: &Cut, window: &mut Window, context: &mut Context<Self>) {
        if let Some(text) = self.document.text().get(self.selection_range())
            && !text.is_empty()
        {
            context.write_to_clipboard(ClipboardItem::new_string(text.to_owned()));
            self.pending_edit_kind = Some(EditKind::Barrier);
            self.replace_text_in_range(None, "", window, context);
        }
    }

    fn insert_newline(
        &mut self,
        _: &InsertNewline,
        window: &mut Window,
        context: &mut Context<Self>,
    ) {
        self.pending_edit_kind = Some(EditKind::Barrier);
        self.replace_text_in_range(None, "\n", window, context);
    }

    fn undo(&mut self, _: &Undo, window: &mut Window, context: &mut Context<Self>) {
        self.finish_composition(context);
        let Some(snapshot) = self.history.undo() else {
            window.play_system_bell();
            return;
        };
        self.restore_snapshot(snapshot, window, context);
    }

    fn redo(&mut self, _: &Redo, window: &mut Window, context: &mut Context<Self>) {
        self.finish_composition(context);
        let Some(snapshot) = self.history.redo() else {
            window.play_system_bell();
            return;
        };
        self.restore_snapshot(snapshot, window, context);
    }

    fn restore_snapshot(
        &mut self,
        snapshot: HistorySnapshot,
        window: &mut Window,
        context: &mut Context<Self>,
    ) {
        let (text, selection) = snapshot.into_parts();
        let batch = self.document.replace_all(text);
        if self.document.apply(batch).is_err()
            || self
                .session
                .set_selection(&self.document, selection)
                .is_err()
        {
            window.play_system_bell();
            return;
        }
        self.viewport.request_caret_visible();
        context.notify();
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        context: &mut Context<Self>,
    ) {
        self.focus_handle.focus(window, context);
        self.is_selecting = true;
        let offset = self.index_for_mouse_position(event.position);
        if event.modifiers.shift {
            self.select_to(offset, context);
        } else {
            self.move_to(offset, context);
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        context: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), context);
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        let Some(layout) = self.last_layout.as_ref() else {
            return 0;
        };
        let offset = match layout.index_for_position(position) {
            Ok(offset) | Err(offset) => offset,
        };
        nearest_grapheme_boundary(self.document.text(), offset.min(self.document.text().len()))
    }

    fn apply_replacement(
        &mut self,
        replacement: Replacement<'_>,
        window: &mut Window,
        context: &mut Context<Self>,
    ) -> bool {
        let batch = EditBatch::single_range(
            self.document.revision(),
            ByteOffset::new(replacement.range.start)..ByteOffset::new(replacement.range.end),
            replacement.text,
        );
        let before = replacement.before.unwrap_or_else(|| self.snapshot());
        if self.document.apply(batch).is_err()
            || self
                .session
                .set_selection(&self.document, replacement.selection)
                .is_err()
        {
            window.play_system_bell();
            return false;
        }
        if let Some(kind) = replacement.kind {
            let after = self.snapshot();
            self.history
                .record(EditTransaction::new(before, after, kind));
        }
        self.viewport.request_caret_visible();
        context.notify();
        true
    }

    fn finish_composition(&mut self, context: &mut Context<Self>) {
        self.session.clear_composition();
        if let Some(before) = self.composition_before.take() {
            let after = self.snapshot();
            self.history
                .record(EditTransaction::new(before, after, EditKind::Barrier));
        }
        context.notify();
    }

    fn highlights(&self) -> Vec<(Range<usize>, HighlightStyle)> {
        let selection = self.selection_range();
        let marked = self
            .session
            .composition()
            .map(CompositionState::marked_range)
            .map(|range| range.start.get()..range.end.get());
        let mut boundaries = vec![0, self.document.text().len()];
        if !selection.is_empty() {
            boundaries.extend([selection.start, selection.end]);
        }
        if let Some(marked) = &marked {
            boundaries.extend([marked.start, marked.end]);
        }
        boundaries.sort_unstable();
        boundaries.dedup();

        boundaries
            .windows(2)
            .filter_map(|pair| {
                let range = pair[0]..pair[1];
                let selected = !selection.is_empty()
                    && range.start >= selection.start
                    && range.end <= selection.end;
                let is_marked = marked
                    .as_ref()
                    .is_some_and(|marked| range.start >= marked.start && range.end <= marked.end);
                (selected || is_marked).then(|| {
                    (
                        range,
                        HighlightStyle {
                            background_color: selected.then_some(hsla(
                                218. / 360.,
                                0.85,
                                0.55,
                                0.38,
                            )),
                            underline: is_marked.then_some(UnderlineStyle {
                                color: Some(hsla(218. / 360., 0.9, 0.7, 1.)),
                                thickness: px(1.),
                                wavy: false,
                            }),
                            ..Default::default()
                        },
                    )
                })
            })
            .collect()
    }
}

impl EntityInputHandler for EditorController {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = utf16_range_to_bytes(self.document.text(), &range_utf16);
        actual_range.replace(byte_range_to_utf16(self.document.text(), &range));
        self.document.text().get(range).map(ToOwned::to_owned)
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let selection = self.session.selection();
        Some(UTF16Selection {
            range: byte_range_to_utf16(self.document.text(), &self.selection_range()),
            reversed: selection.is_reversed(),
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.session
            .composition()
            .map(CompositionState::marked_range)
            .map(|range| range.start.get()..range.end.get())
            .map(|range| byte_range_to_utf16(self.document.text(), &range))
    }

    fn unmark_text(&mut self, _: &mut Window, context: &mut Context<Self>) {
        self.finish_composition(context);
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        context: &mut Context<Self>,
    ) {
        let text = normalize_input_text(text);
        let composition = self
            .session
            .composition()
            .map(CompositionState::marked_range);
        let range = composition
            .as_ref()
            .map(|range| range.start.get()..range.end.get())
            .or_else(|| {
                range_utf16
                    .as_ref()
                    .map(|range| utf16_range_to_bytes(self.document.text(), range))
            })
            .unwrap_or_else(|| self.selection_range());
        let cursor = range.start + text.len();
        let kind = self.pending_edit_kind.take().unwrap_or_else(|| {
            if composition.is_some() || range_utf16.is_some() {
                EditKind::Barrier
            } else {
                EditKind::Typing
            }
        });
        let before = self
            .composition_before
            .take()
            .or_else(|| self.pending_history_before.take());
        self.session.clear_composition();
        self.apply_replacement(
            Replacement {
                range,
                text: &text,
                selection: Selection::caret(ByteOffset::new(cursor)),
                kind: Some(kind),
                before,
            },
            window,
            context,
        );
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        context: &mut Context<Self>,
    ) {
        let text = normalize_input_text(text);
        let existing_marked = self
            .session
            .composition()
            .map(CompositionState::marked_range);
        let range = match (existing_marked.as_ref(), range_utf16.as_ref()) {
            (Some(marked), Some(range)) => {
                let marked = marked.start.get()..marked.end.get();
                utf16_range_to_bytes_in(self.document.text(), &marked, range)
            }
            (Some(marked), None) => marked.start.get()..marked.end.get(),
            (None, Some(range)) => utf16_range_to_bytes(self.document.text(), range),
            (None, None) => self.selection_range(),
        };
        if self.composition_before.is_none() {
            self.composition_before = Some(self.snapshot());
        }
        let selected = new_selected_range_utf16
            .as_ref()
            .map(|range| utf16_range_to_bytes(&text, range))
            .map(|relative| range.start + relative.start..range.start + relative.end)
            .unwrap_or_else(|| range.start + text.len()..range.start + text.len());
        let selection = Selection::new(
            ByteOffset::new(selected.start),
            ByteOffset::new(selected.end),
        );
        let token = if let Some(composition) = self.session.composition() {
            composition.transaction()
        } else {
            let token = TransactionToken::new(self.next_transaction_token);
            self.next_transaction_token = self.next_transaction_token.saturating_add(1);
            token
        };
        if !self.apply_replacement(
            Replacement {
                range: range.clone(),
                text: &text,
                selection,
                kind: None,
                before: None,
            },
            window,
            context,
        ) {
            self.composition_before = None;
            return;
        }
        let marked = range.start..range.start + text.len();
        if text.is_empty() {
            self.session.clear_composition();
        } else if self
            .session
            .set_composition(
                &self.document,
                CompositionState::new(
                    ByteOffset::new(marked.start)..ByteOffset::new(marked.end),
                    token,
                ),
            )
            .is_err()
        {
            self.composition_before = None;
            window.play_system_bell();
        }
        context.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let layout = self.last_layout.as_ref()?;
        let range = utf16_range_to_bytes(self.document.text(), &range_utf16);
        let position = layout.position_for_index(range.start)?;
        Some(Bounds::new(position, size(px(1.), layout.line_height())))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let layout = self.last_layout.as_ref()?;
        let offset = match layout.index_for_position(point) {
            Ok(offset) | Err(offset) => offset,
        };
        Some(byte_to_utf16_offset(
            self.document.text(),
            nearest_grapheme_boundary(self.document.text(), offset.min(self.document.text().len())),
        ))
    }

    fn set_selected_text_range(
        &mut self,
        range_utf16: Range<usize>,
        _: &mut Window,
        context: &mut Context<Self>,
    ) {
        let range = utf16_range_to_bytes(self.document.text(), &range_utf16);
        self.set_selection(
            Selection::new(
                ByteOffset::new(nearest_grapheme_boundary(self.document.text(), range.start)),
                ByteOffset::new(nearest_grapheme_boundary(self.document.text(), range.end)),
            ),
            context,
        );
    }

    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(byte_to_utf16_offset(
            self.document.text(),
            self.document.text().len(),
        ))
    }
}

impl Focusable for EditorController {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EditorController {
    fn render(&mut self, window: &mut Window, context: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(window);
        let display_text: SharedString = if self.document.is_empty() {
            "Start writing C…".into()
        } else {
            self.document.text().to_owned().into()
        };
        let text = if self.document.is_empty() {
            StyledText::new(display_text).with_highlights([(
                0.."Start writing C…".len(),
                HighlightStyle {
                    color: Some(hsla(0., 0., 0.55, 1.)),
                    ..Default::default()
                },
            )])
        } else {
            StyledText::new(display_text).with_highlights(self.highlights())
        };
        let current_line = line_and_column(self.document.text(), self.cursor_offset()).0;
        let scroll_offset = self.viewport.scroll_offset();
        let gutter = (0..self.document.line_count()).fold(
            div()
                .relative()
                .top(EDITOR_PADDING + scroll_offset.y)
                .w_full()
                .flex()
                .flex_col(),
            |gutter, line| {
                gutter.child(
                    div()
                        .h(LINE_HEIGHT)
                        .w_full()
                        .pr_3()
                        .flex()
                        .items_center()
                        .justify_end()
                        .bg(if line == current_line {
                            rgb(0x1a2632)
                        } else {
                            rgb(0x111821)
                        })
                        .text_color(if line == current_line {
                            rgb(0xe7edf4)
                        } else {
                            rgb(0x667586)
                        })
                        .child((line + 1).to_string()),
                )
            },
        );

        div()
            .id("luma-source-editor")
            .key_context("LumaEditor")
            .role(Role::MultilineTextInput)
            .aria_label("Source editor")
            .aria_value(self.document.text().to_owned())
            .aria_placeholder("Start writing C…")
            .track_focus(&self.focus_handle)
            .cursor(CursorStyle::IBeam)
            .on_action(context.listener(Self::backspace))
            .on_action(context.listener(Self::delete))
            .on_action(context.listener(Self::left))
            .on_action(context.listener(Self::right))
            .on_action(context.listener(Self::up))
            .on_action(context.listener(Self::down))
            .on_action(context.listener(Self::select_left))
            .on_action(context.listener(Self::select_right))
            .on_action(context.listener(Self::select_up))
            .on_action(context.listener(Self::select_down))
            .on_action(context.listener(Self::select_all))
            .on_action(context.listener(Self::home))
            .on_action(context.listener(Self::end))
            .on_action(context.listener(Self::undo))
            .on_action(context.listener(Self::redo))
            .on_action(context.listener(Self::paste))
            .on_action(context.listener(Self::cut))
            .on_action(context.listener(Self::copy))
            .on_action(context.listener(Self::insert_newline))
            .on_mouse_down(MouseButton::Left, context.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, context.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, context.listener(Self::on_mouse_up))
            .on_mouse_move(context.listener(Self::on_mouse_move))
            .size_full()
            .flex()
            .overflow_hidden()
            .border_1()
            .border_color(if focused {
                rgb(0x24c8db)
            } else {
                rgb(0x263241)
            })
            .bg(rgb(0x0b0f14))
            .font_family("Menlo")
            .text_size(px(15.))
            .line_height(LINE_HEIGHT)
            .child(
                div()
                    .w(GUTTER_WIDTH)
                    .h_full()
                    .flex_none()
                    .overflow_hidden()
                    .border_r_1()
                    .border_color(rgb(0x263241))
                    .bg(rgb(0x111821))
                    .text_size(px(12.))
                    .child(gutter),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .child(
                        div()
                            .absolute()
                            .top(
                                EDITOR_PADDING
                                    + scroll_offset.y
                                    + LINE_HEIGHT * current_line as f32,
                            )
                            .left_0()
                            .right_0()
                            .h(LINE_HEIGHT)
                            .bg(rgb(0x0f1821)),
                    )
                    .child(
                        div()
                            .size_full()
                            .p_2()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(EditorTextElement {
                                editor: context.entity(),
                                text,
                            }),
                    ),
            )
    }
}

struct EditorTextElement {
    editor: Entity<EditorController>,
    text: StyledText,
}

struct EditorPrepaintState {
    layout: TextLayout,
    cursor: Option<PaintQuad>,
    viewport: Bounds<Pixels>,
    scroll_offset: Point<Pixels>,
    caret_revealed: bool,
}

impl IntoElement for EditorTextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for EditorTextElement {
    type RequestLayoutState = ();
    type PrepaintState = EditorPrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        context: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.text.request_layout(id, inspector_id, window, context)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        context: &mut App,
    ) -> Self::PrepaintState {
        let viewport = window.content_mask().bounds;
        let (cursor_offset, focused, selection_empty, old_scroll, reveal) = {
            let editor = self.editor.read(context);
            (
                editor.cursor_offset(),
                editor.focus_handle.is_focused(window),
                editor.session.selection().is_empty(),
                editor.viewport.scroll_offset(),
                editor
                    .viewport
                    .needs_caret_reveal(editor.document.revision().get(), viewport.size),
            )
        };
        let shifted_bounds = Bounds::new(bounds.origin + old_scroll, bounds.size);
        self.text.prepaint(
            id,
            inspector_id,
            shifted_bounds,
            request_layout,
            window,
            context,
        );

        let first_layout = self.text.layout().clone();
        let line_height = first_layout.line_height();
        let mut content_width = Pixels::ZERO;
        let mut content_height = Pixels::ZERO;
        for line in first_layout.line_layouts() {
            let line_size = line.size(line_height);
            content_width = content_width.max(line_size.width);
            content_height += line_size.height;
        }
        content_width += EDITOR_PADDING * 2. + px(1.5);
        content_height += EDITOR_PADDING * 2.;
        let max_scroll = point(
            (content_width - viewport.size.width).max(Pixels::ZERO),
            (content_height - viewport.size.height).max(Pixels::ZERO),
        );
        let clamped_scroll = point(
            old_scroll.x.clamp(-max_scroll.x, Pixels::ZERO),
            old_scroll.y.clamp(-max_scroll.y, Pixels::ZERO),
        );
        let initial_caret = first_layout
            .position_for_index(cursor_offset)
            .map(|position| Bounds::new(position, size(px(1.5), line_height)))
            .map(|mut caret| {
                caret.origin += clamped_scroll - old_scroll;
                caret
            });
        let caret_revealed = focused
            && reveal
            && viewport.size.width > Pixels::ZERO
            && viewport.size.height > Pixels::ZERO;
        let scroll_offset = if caret_revealed {
            initial_caret
                .map(|caret| reveal_scroll(clamped_scroll, caret, viewport, max_scroll))
                .unwrap_or(clamped_scroll)
        } else {
            clamped_scroll
        };
        if scroll_offset != old_scroll {
            self.text.prepaint(
                id,
                inspector_id,
                Bounds::new(bounds.origin + scroll_offset, bounds.size),
                request_layout,
                window,
                context,
            );
        }

        let layout = self.text.layout().clone();
        let cursor = if focused && selection_empty {
            layout
                .position_for_index(cursor_offset)
                .map(|position| Bounds::new(position, size(px(1.5), layout.line_height())))
                .map(|bounds| fill(bounds, rgb(0x24c8db)))
        } else {
            None
        };
        EditorPrepaintState {
            layout,
            cursor,
            viewport,
            scroll_offset,
            caret_revealed,
        }
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        context: &mut App,
    ) {
        let focus_handle = self.editor.read(context).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(prepaint.viewport, self.editor.clone()),
            context,
        );
        self.text.paint(
            id,
            inspector_id,
            bounds,
            request_layout,
            &mut (),
            window,
            context,
        );
        if let Some(cursor) = prepaint.cursor.take() {
            window.paint_quad(cursor);
        }
        self.editor.update(context, |editor, context| {
            editor.last_layout = Some(prepaint.layout.clone());
            if editor.viewport.finish_layout(
                editor.document.revision().get(),
                prepaint.viewport.size,
                prepaint.scroll_offset,
                prepaint.caret_revealed,
            ) {
                context.notify();
            }
        });
        window.invalidate_character_coordinates();
    }
}

fn normalize_input_text(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn line_and_column(text: &str, offset: usize) -> (usize, usize) {
    let offset = nearest_grapheme_boundary(text, offset);
    let line_start = text[..offset].rfind('\n').map_or(0, |index| index + 1);
    let line = text[..line_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    let column = text[line_start..offset].graphemes(true).count();
    (line, column)
}

fn line_content_end(text: &str, line: usize) -> usize {
    let start = line_start(text, line);
    text[start..]
        .find('\n')
        .map_or(text.len(), |length| start + length)
}

fn line_start(text: &str, line: usize) -> usize {
    if line == 0 {
        return 0;
    }
    text.match_indices('\n')
        .nth(line - 1)
        .map_or(text.len(), |(index, _)| index + 1)
}

fn offset_for_line_column(text: &str, line: usize, column: usize) -> usize {
    let start = line_start(text, line);
    let end = line_content_end(text, line);
    text[start..end]
        .grapheme_indices(true)
        .nth(column)
        .map_or(end, |(index, _)| start + index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_platform_line_endings() {
        assert_eq!(normalize_input_text("a\r\nb\rc"), "a\nb\nc");
    }

    #[test]
    fn resolves_unicode_line_columns() {
        let text = "a😀\n中e\u{301}\n";
        assert_eq!(line_and_column(text, 5), (0, 2));
        assert_eq!(line_and_column(text, 9), (1, 1));
        assert_eq!(offset_for_line_column(text, 1, 2), 12);
        assert_eq!(offset_for_line_column(text, 1, 99), 12);
    }
}
