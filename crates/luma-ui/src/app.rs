use crate::{
    FocusNext, FocusPrevious,
    text_input::TextInput,
    view::{landing, workbench},
};
use gpui::{App, Context, Entity, FocusHandle, Focusable, Keystroke, Render, Window, prelude::*};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SidebarSection {
    Explorer,
    Search,
    SourceControl,
    Run,
    Extensions,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum WorkspaceTab {
    Welcome,
    Editor,
}

pub(super) struct LumaApp {
    pub(super) focus: FocusHandle,
    pub(super) command_input: Entity<TextInput>,
    pub(super) source_input: Entity<TextInput>,
    pub(super) active_sidebar: SidebarSection,
    pub(super) active_tab: WorkspaceTab,
    pub(super) editor_open: bool,
    pub(super) show_landing: bool,
    pub(super) recent_keystrokes: Vec<Keystroke>,
    pub(super) activations: usize,
    pub(super) status: String,
}

impl LumaApp {
    pub(super) fn new(window: &mut Window, context: &mut Context<Self>) -> Self {
        let focus = context.focus_handle();
        let command_input = context
            .new(|context| TextInput::single_line("Search files or type a filename", context));
        let source_input = context.new(|context| {
            TextInput::multiline_with_text(
                "Start writing C…",
                "#include <stdio.h>\n\nint main(void) {\n    printf(\"Hello from Luma!\\n\");\n    return 0;\n}\n",
                context,
            )
        });
        window.focus(&focus, context);
        Self {
            focus,
            command_input,
            source_input,
            active_sidebar: SidebarSection::Explorer,
            active_tab: WorkspaceTab::Welcome,
            editor_open: false,
            show_landing: true,
            recent_keystrokes: Vec::new(),
            activations: 0,
            status: "Ready · Zig 0.16.0".to_owned(),
        }
    }

    pub(super) fn focus_next(
        &mut self,
        _: &FocusNext,
        window: &mut Window,
        context: &mut Context<Self>,
    ) {
        window.focus_next(context);
    }

    pub(super) fn focus_previous(
        &mut self,
        _: &FocusPrevious,
        window: &mut Window,
        context: &mut Context<Self>,
    ) {
        window.focus_prev(context);
    }

    pub(super) fn open_editor(
        &mut self,
        message: &str,
        window: &mut Window,
        context: &mut Context<Self>,
    ) {
        self.show_landing = false;
        self.editor_open = true;
        self.active_tab = WorkspaceTab::Editor;
        self.active_sidebar = SidebarSection::Explorer;
        self.status = message.to_owned();
        window.focus(&self.source_input.read(context).focus_handle(), context);
        context.notify();
    }

    pub(super) fn show_welcome(&mut self, message: &str, context: &mut Context<Self>) {
        self.show_landing = false;
        self.active_tab = WorkspaceTab::Welcome;
        self.status = message.to_owned();
        context.notify();
    }

    pub(super) fn select_sidebar(
        &mut self,
        section: SidebarSection,
        message: &str,
        context: &mut Context<Self>,
    ) {
        self.active_sidebar = section;
        self.status = message.to_owned();
        context.notify();
    }

    pub(super) fn select_search(&mut self, window: &mut Window, context: &mut Context<Self>) {
        self.active_sidebar = SidebarSection::Search;
        self.status = "Search files".to_owned();
        window.focus(&self.command_input.read(context).focus_handle(), context);
        context.notify();
    }

    pub(super) fn set_status(&mut self, message: &str, context: &mut Context<Self>) {
        self.status = message.to_owned();
        context.notify();
    }

    pub(super) fn run(&mut self, context: &mut Context<Self>) {
        self.activations += 1;
        self.status = format!("Build succeeded · Run #{} · 0 errors", self.activations);
        context.notify();
    }

    pub(super) fn record_keystroke(&mut self, keystroke: Keystroke, context: &mut Context<Self>) {
        self.recent_keystrokes.push(keystroke);
        if self.recent_keystrokes.len() > 16 {
            self.recent_keystrokes.remove(0);
        }
        context.notify();
    }
}

impl Focusable for LumaApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for LumaApp {
    fn render(&mut self, _: &mut Window, context: &mut Context<Self>) -> impl IntoElement {
        if self.show_landing {
            landing::render(context).into_any_element()
        } else {
            workbench::render(self, context).into_any_element()
        }
    }
}
