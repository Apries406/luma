// Portions adapted from GPUI's `examples/input.rs` at Zed commit
// 251854020a9dbea1a388bf392c9bd04fd136a557.
// Copyright 2022-2025 Zed Industries, Inc. Licensed under Apache-2.0.
// Modified by Lec for Luma's UTF-8, multiline, IME, and accessibility spike.

use std::ops::Range;

use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, FocusHandle, Focusable, GlobalElementId, HighlightStyle, KeyBinding,
    LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point,
    Role, SharedString, StyledText, TextLayout, UTF16Selection, UnderlineStyle, Window, actions,
    div, fill, hsla, point, prelude::*, px, size,
};
use unicode_segmentation::UnicodeSegmentation;

actions!(
    luma_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        Paste,
        Cut,
        Copy,
        InsertNewline,
    ]
);

pub fn register_key_bindings(cx: &mut App) {
    let context = Some("LumaInput");
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, context),
        KeyBinding::new("delete", Delete, context),
        KeyBinding::new("left", Left, context),
        KeyBinding::new("right", Right, context),
        KeyBinding::new("shift-left", SelectLeft, context),
        KeyBinding::new("shift-right", SelectRight, context),
        KeyBinding::new("home", Home, context),
        KeyBinding::new("end", End, context),
        KeyBinding::new("cmd-a", SelectAll, context),
        KeyBinding::new("ctrl-a", SelectAll, context),
        KeyBinding::new("cmd-c", Copy, context),
        KeyBinding::new("ctrl-c", Copy, context),
        KeyBinding::new("cmd-x", Cut, context),
        KeyBinding::new("ctrl-x", Cut, context),
        KeyBinding::new("cmd-v", Paste, context),
        KeyBinding::new("ctrl-v", Paste, context),
        KeyBinding::new("enter", InsertNewline, context),
    ]);
}

pub struct TextInput {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<TextLayout>,
    scroll_offset: Point<Pixels>,
    is_selecting: bool,
    multiline: bool,
}

impl TextInput {
    pub fn single_line(placeholder: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self::new(placeholder.into(), false, cx)
    }

    pub fn multiline_with_text(
        placeholder: impl Into<SharedString>,
        content: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut input = Self::new(placeholder.into(), true, cx);
        input.content = content.into();
        input.selected_range = input.content.len()..input.content.len();
        input
    }

    fn new(placeholder: SharedString, multiline: bool, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle().tab_stop(true),
            content: "".into(),
            placeholder,
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            scroll_offset: Point::default(),
            is_selecting: false,
            multiline,
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(previous_boundary(&self.content, self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx);
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(next_boundary(&self.content, self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.end, cx);
        }
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(previous_boundary(&self.content, self.cursor_offset()), cx);
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(next_boundary(&self.content, self.cursor_offset()), cx);
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx);
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let previous = previous_boundary(&self.content, self.cursor_offset());
            if self.cursor_offset() == previous {
                window.play_system_bell();
                return;
            }
            self.select_to(previous, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let next = next_boundary(&self.content, self.cursor_offset());
            if self.cursor_offset() == next {
                window.play_system_bell();
                return;
            }
            self.select_to(next, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            <Self as EntityInputHandler>::paste(self, item, window, cx);
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = self.content.get(self.selected_range.clone())
            && !text.is_empty()
        {
            cx.write_to_clipboard(ClipboardItem::new_string(text.to_owned()));
        }
    }

    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = self.content.get(self.selected_range.clone())
            && !text.is_empty()
        {
            cx.write_to_clipboard(ClipboardItem::new_string(text.to_owned()));
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    fn insert_newline(&mut self, _: &InsertNewline, window: &mut Window, cx: &mut Context<Self>) {
        if self.multiline {
            self.replace_text_in_range(None, "\n", window, cx);
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_handle.focus(window, cx);
        self.is_selecting = true;
        let offset = self.index_for_mouse_position(event.position);
        if event.modifiers.shift {
            self.select_to(offset, cx);
        } else {
            self.move_to(offset, cx);
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = nearest_grapheme_boundary(&self.content, offset);
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        self.marked_range = None;
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = nearest_grapheme_boundary(&self.content, offset);
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        self.marked_range = None;
        cx.notify();
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        let Some(layout) = self.last_layout.as_ref() else {
            return 0;
        };
        let offset = match layout.index_for_position(position) {
            Ok(offset) | Err(offset) => offset,
        };
        nearest_grapheme_boundary(&self.content, offset.min(self.content.len()))
    }

    fn input_text(&self, text: &str) -> String {
        if self.multiline {
            text.to_owned()
        } else {
            text.replace(['\r', '\n'], " ")
        }
    }

    fn highlights(&self) -> Vec<(Range<usize>, HighlightStyle)> {
        let mut boundaries = vec![0, self.content.len()];
        if !self.selected_range.is_empty() {
            boundaries.extend([self.selected_range.start, self.selected_range.end]);
        }
        if let Some(marked) = &self.marked_range {
            boundaries.extend([marked.start, marked.end]);
        }
        boundaries.sort_unstable();
        boundaries.dedup();

        boundaries
            .windows(2)
            .filter_map(|pair| {
                let range = pair[0]..pair[1];
                let selected = !self.selected_range.is_empty()
                    && range.start >= self.selected_range.start
                    && range.end <= self.selected_range.end;
                let marked = self
                    .marked_range
                    .as_ref()
                    .is_some_and(|marked| range.start >= marked.start && range.end <= marked.end);
                (selected || marked).then(|| {
                    (
                        range,
                        HighlightStyle {
                            background_color: selected.then_some(hsla(
                                218. / 360.,
                                0.85,
                                0.55,
                                0.38,
                            )),
                            underline: marked.then_some(UnderlineStyle {
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

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = range_from_utf16(&self.content, &range_utf16);
        actual_range.replace(range_to_utf16(&self.content, &range));
        self.content.get(range).map(ToOwned::to_owned)
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: range_to_utf16(&self.content, &self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| range_to_utf16(&self.content, range))
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = replacement_range_for_commit(
            &self.content,
            self.marked_range.as_ref(),
            &self.selected_range,
            range_utf16.as_ref(),
        );
        let text = self.input_text(text);
        let mut content = self.content.to_string();
        content.replace_range(range.clone(), &text);
        let cursor = range.start + text.len();
        self.content = content.into();
        self.selected_range = cursor..cursor;
        self.selection_reversed = false;
        self.marked_range = None;
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = replacement_range_for_composition(
            &self.content,
            self.marked_range.as_ref(),
            &self.selected_range,
            range_utf16.as_ref(),
        );
        let text = self.input_text(text);
        let mut content = self.content.to_string();
        content.replace_range(range.clone(), &text);

        self.marked_range = (!text.is_empty()).then_some(range.start..range.start + text.len());
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|selection| range_from_utf16(&text, selection))
            .map(|selection| range.start + selection.start..range.start + selection.end)
            .unwrap_or_else(|| range.start + text.len()..range.start + text.len());
        self.selection_reversed = false;
        self.content = content.into();
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let layout = self.last_layout.as_ref()?;
        let range = range_from_utf16(&self.content, &range_utf16);
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
        Some(offset_to_utf16(
            &self.content,
            nearest_grapheme_boundary(&self.content, offset.min(self.content.len())),
        ))
    }

    fn set_selected_text_range(
        &mut self,
        range_utf16: Range<usize>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_from_utf16(&self.content, &range_utf16);
        let start = nearest_grapheme_boundary(&self.content, range.start);
        let end = nearest_grapheme_boundary(&self.content, range.end);
        self.selected_range = start.min(end)..start.max(end);
        self.selection_reversed = start > end;
        self.marked_range = None;
        cx.notify();
    }

    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(offset_to_utf16(&self.content, self.content.len()))
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(window);
        let display_text = if self.content.is_empty() {
            self.placeholder.clone()
        } else {
            self.content.clone()
        };
        let text = if self.content.is_empty() {
            StyledText::new(display_text).with_highlights([(
                0..self.placeholder.len(),
                HighlightStyle {
                    color: Some(hsla(0., 0., 0.55, 1.)),
                    ..Default::default()
                },
            )])
        } else {
            StyledText::new(display_text).with_highlights(self.highlights())
        };

        div()
            .id(if self.multiline {
                "luma-multiline-input"
            } else {
                "luma-single-line-input"
            })
            .key_context("LumaInput")
            .role(if self.multiline {
                Role::MultilineTextInput
            } else {
                Role::TextInput
            })
            .aria_label(if self.multiline {
                "Source editor"
            } else {
                "File search"
            })
            .aria_value(self.content.clone())
            .aria_placeholder(self.placeholder.clone())
            .track_focus(&self.focus_handle)
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::insert_newline))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .w_full()
            .flex_shrink_0()
            .when(self.multiline, |element| {
                element
                    .h_full()
                    .min_h(px(128.))
                    .font_family("Menlo")
                    .whitespace_nowrap()
            })
            .when(!self.multiline, |element| {
                element.h(px(40.)).whitespace_nowrap()
            })
            .p_2()
            .overflow_hidden()
            .rounded_md()
            .border_1()
            .border_color(if focused {
                gpui::blue()
            } else {
                hsla(0., 0., 0.25, 1.)
            })
            .bg(gpui::rgb(0x20232b))
            .text_color(gpui::rgb(0xe6e8ef))
            .text_size(px(15.))
            .line_height(px(22.))
            .child(div().size_full().overflow_hidden().child(InputTextElement {
                input: cx.entity(),
                text,
            }))
    }
}

struct InputTextElement {
    input: Entity<TextInput>,
    text: StyledText,
}

struct InputPrepaintState {
    layout: TextLayout,
    cursor: Option<PaintQuad>,
    viewport: Bounds<Pixels>,
    scroll_offset: Point<Pixels>,
    scroll_changed: bool,
}

impl IntoElement for InputTextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for InputTextElement {
    type RequestLayoutState = ();
    type PrepaintState = InputPrepaintState;

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
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.text.request_layout(id, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let viewport = window.content_mask().bounds;
        let (cursor_offset, focused, selection_empty, old_scroll) = {
            let input = self.input.read(cx);
            (
                input.cursor_offset(),
                input.focus_handle.is_focused(window),
                input.selected_range.is_empty(),
                input.scroll_offset,
            )
        };
        let shifted_bounds = Bounds::new(bounds.origin + old_scroll, bounds.size);
        self.text
            .prepaint(id, inspector_id, shifted_bounds, request_layout, window, cx);

        let first_layout = self.text.layout().clone();
        let line_height = first_layout.line_height();
        let mut content_width = Pixels::ZERO;
        let mut content_height = Pixels::ZERO;
        for line in first_layout.line_layouts() {
            let line_size = line.size(line_height);
            content_width = content_width.max(line_size.width);
            content_height += line_size.height;
        }
        content_width += px(1.5);
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
        let scroll_offset =
            if focused && viewport.size.width > Pixels::ZERO && viewport.size.height > Pixels::ZERO
            {
                initial_caret
                    .map(|caret| reveal_scroll(clamped_scroll, caret, viewport, max_scroll))
                    .unwrap_or(clamped_scroll)
            } else {
                clamped_scroll
            };
        let scroll_changed = scroll_offset != old_scroll;
        if scroll_changed {
            self.text.prepaint(
                id,
                inspector_id,
                Bounds::new(bounds.origin + scroll_offset, bounds.size),
                request_layout,
                window,
                cx,
            );
        }

        let layout = self.text.layout().clone();
        let caret_bounds = layout
            .position_for_index(cursor_offset)
            .map(|position| Bounds::new(position, size(px(1.5), layout.line_height())));
        let cursor = if focused && selection_empty {
            caret_bounds.map(|bounds| fill(bounds, gpui::blue()))
        } else {
            None
        };
        InputPrepaintState {
            layout,
            cursor,
            viewport,
            scroll_offset,
            scroll_changed,
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
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(prepaint.viewport, self.input.clone()),
            cx,
        );
        self.text.paint(
            id,
            inspector_id,
            bounds,
            request_layout,
            &mut (),
            window,
            cx,
        );
        if let Some(cursor) = prepaint.cursor.take() {
            window.paint_quad(cursor);
        }
        self.input.update(cx, |input, _| {
            input.last_layout = Some(prepaint.layout.clone());
            input.scroll_offset = prepaint.scroll_offset;
        });
        if prepaint.scroll_changed {
            window.invalidate_character_coordinates();
        }
    }
}

fn reveal_scroll(
    offset: Point<Pixels>,
    target: Bounds<Pixels>,
    viewport: Bounds<Pixels>,
    max_scroll: Point<Pixels>,
) -> Point<Pixels> {
    let reveal_axis = |offset: Pixels,
                       target_start: Pixels,
                       target_end: Pixels,
                       viewport_start: Pixels,
                       viewport_end: Pixels,
                       max_scroll: Pixels| {
        let offset = if target_start < viewport_start {
            offset + viewport_start - target_start
        } else if target_end > viewport_end {
            offset - (target_end - viewport_end)
        } else {
            offset
        };
        offset.clamp(-max_scroll, Pixels::ZERO)
    };

    point(
        reveal_axis(
            offset.x,
            target.left(),
            target.right(),
            viewport.left(),
            viewport.right(),
            max_scroll.x,
        ),
        reveal_axis(
            offset.y,
            target.top(),
            target.bottom(),
            viewport.top(),
            viewport.bottom(),
            max_scroll.y,
        ),
    )
}

fn offset_from_utf16(content: &str, offset: usize) -> usize {
    let mut utf8_offset = 0;
    let mut utf16_count = 0;
    for character in content.chars() {
        if utf16_count >= offset {
            break;
        }
        utf16_count += character.len_utf16();
        utf8_offset += character.len_utf8();
    }
    utf8_offset
}

fn offset_from_utf16_left(content: &str, offset: usize) -> usize {
    let mut utf8_offset = 0;
    let mut utf16_count = 0;
    for character in content.chars() {
        let next_utf16_count = utf16_count + character.len_utf16();
        if offset < next_utf16_count {
            break;
        }
        utf16_count = next_utf16_count;
        utf8_offset += character.len_utf8();
    }
    utf8_offset
}

fn offset_to_utf16(content: &str, offset: usize) -> usize {
    let mut utf16_offset = 0;
    let mut utf8_count = 0;
    for character in content.chars() {
        if utf8_count >= offset {
            break;
        }
        utf8_count += character.len_utf8();
        utf16_offset += character.len_utf16();
    }
    utf16_offset
}

fn range_to_utf16(content: &str, range: &Range<usize>) -> Range<usize> {
    offset_to_utf16(content, range.start)..offset_to_utf16(content, range.end)
}

fn range_from_utf16(content: &str, range: &Range<usize>) -> Range<usize> {
    if range.is_empty() {
        let offset = offset_from_utf16(content, range.start);
        return offset..offset;
    }
    offset_from_utf16_left(content, range.start)..offset_from_utf16(content, range.end)
}

fn range_from_utf16_in_range(
    content: &str,
    base: &Range<usize>,
    range: &Range<usize>,
) -> Range<usize> {
    let relative = range_from_utf16(&content[base.clone()], range);
    base.start + relative.start..base.start + relative.end
}

fn replacement_range_for_commit(
    content: &str,
    marked: Option<&Range<usize>>,
    selected: &Range<usize>,
    range_utf16: Option<&Range<usize>>,
) -> Range<usize> {
    marked
        .cloned()
        .or_else(|| range_utf16.map(|range| range_from_utf16(content, range)))
        .unwrap_or_else(|| selected.clone())
}

fn replacement_range_for_composition(
    content: &str,
    marked: Option<&Range<usize>>,
    selected: &Range<usize>,
    range_utf16: Option<&Range<usize>>,
) -> Range<usize> {
    match (marked, range_utf16) {
        (Some(marked), Some(range)) => range_from_utf16_in_range(content, marked, range),
        (Some(marked), None) => marked.clone(),
        (None, Some(range)) => range_from_utf16(content, range),
        (None, None) => selected.clone(),
    }
}

fn previous_boundary(content: &str, offset: usize) -> usize {
    content
        .grapheme_indices(true)
        .rev()
        .find_map(|(index, _)| (index < offset).then_some(index))
        .unwrap_or(0)
}

fn next_boundary(content: &str, offset: usize) -> usize {
    content
        .grapheme_indices(true)
        .find_map(|(index, _)| (index > offset).then_some(index))
        .unwrap_or(content.len())
}

fn nearest_grapheme_boundary(content: &str, offset: usize) -> usize {
    let offset = offset.min(content.len());
    if offset == content.len()
        || content
            .grapheme_indices(true)
            .any(|(index, _)| index == offset)
    {
        return offset;
    }
    let previous = previous_boundary(content, offset + 1);
    let next = next_boundary(content, offset);
    if offset - previous <= next - offset {
        previous
    } else {
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_utf8_and_utf16_offsets_without_splitting_scalars() {
        let content = "a😀e\u{301}中";
        for (utf8, utf16) in [(0, 0), (1, 1), (5, 3), (8, 5), (11, 6)] {
            assert_eq!(offset_to_utf16(content, utf8), utf16);
            assert_eq!(offset_from_utf16(content, utf16), utf8);
        }
        assert_eq!(offset_from_utf16(content, 2), 5);
        assert_eq!(offset_to_utf16(content, 2), 3);
        assert_eq!(range_from_utf16(content, &(1..5)), 1..8);
        assert_eq!(range_to_utf16(content, &(1..8)), 1..5);
    }

    #[test]
    fn expands_utf16_ranges_that_split_surrogate_pairs() {
        let content = "a😀b";
        assert_eq!(range_from_utf16(content, &(2..3)), 1..5);
        assert_eq!(range_from_utf16(content, &(2..2)), 5..5);
    }

    #[test]
    fn resolves_ime_replacement_ranges_relative_to_marked_text() {
        let content = "prefixa😀b";
        let marked = 6..12;
        let selected = 12..12;
        assert_eq!(
            replacement_range_for_composition(content, Some(&marked), &selected, Some(&(2..3))),
            7..11
        );
        assert_eq!(
            replacement_range_for_commit(content, Some(&marked), &selected, Some(&(0..1))),
            marked
        );
        assert_eq!(
            replacement_range_for_composition(content, None, &selected, Some(&(0..1))),
            0..1
        );
        assert_eq!(
            replacement_range_for_commit(content, None, &selected, None),
            selected
        );
    }

    #[test]
    fn reveal_scrolls_caret_minimally_on_both_axes() {
        let viewport = Bounds::new(point(px(10.), px(20.)), size(px(100.), px(40.)));
        let max_scroll = point(px(50.), px(30.));

        assert_eq!(
            reveal_scroll(
                point(px(-20.), px(-10.)),
                Bounds::new(point(px(125.), px(68.)), size(px(2.), px(22.))),
                viewport,
                max_scroll,
            ),
            point(px(-37.), px(-30.))
        );
        assert_eq!(
            reveal_scroll(
                point(px(-20.), px(-10.)),
                Bounds::new(point(px(40.), px(30.)), size(px(2.), px(12.))),
                viewport,
                max_scroll,
            ),
            point(px(-20.), px(-10.))
        );
    }

    #[test]
    fn navigates_extended_grapheme_boundaries() {
        let content = "a\r\n👩‍💻e\u{301}中";
        let boundaries: Vec<_> = content
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .chain([content.len()])
            .collect();
        assert_eq!(boundaries, [0, 1, 3, 14, 17, 20]);
        assert_eq!(previous_boundary(content, 14), 3);
        assert_eq!(next_boundary(content, 3), 14);
        assert_eq!(nearest_grapheme_boundary(content, 8), 3);
        assert_eq!(nearest_grapheme_boundary(content, 12), 14);
    }
}
