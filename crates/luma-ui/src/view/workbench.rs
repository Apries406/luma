use crate::{
    Activate,
    app::{LumaApp, SidebarSection, WorkspaceTab},
};
use gpui::{Context, FontWeight, IntoElement, Keystroke, Role, div, prelude::*, px, rgb};

const BACKGROUND: u32 = 0x0b0f14;
const SURFACE: u32 = 0x111821;
const SURFACE_RAISED: u32 = 0x17212c;
const BORDER: u32 = 0x263241;
const TEXT: u32 = 0xe7edf4;
const MUTED: u32 = 0x8d9aaa;
const CYAN: u32 = 0x24c8db;
const ORANGE: u32 = 0xf0a14a;

fn sidebar(app: &LumaApp, context: &mut Context<LumaApp>) -> impl IntoElement {
    let content = match app.active_sidebar {
        SidebarSection::Explorer => div()
            .child(
                div()
                    .px_3()
                    .py_2()
                    .text_size(px(11.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(MUTED))
                    .child("EXPLORER"),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .text_size(px(12.))
                    .font_weight(FontWeight::BOLD)
                    .child("⌄  HELLO-C"),
            )
            .child(
                div()
                    .id("explorer-main-c")
                    .px_4()
                    .py_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(SURFACE_RAISED)))
                    .on_click(context.listener(|app, _, window, context| {
                        app.open_editor("Opened hello-c/main.c", window, context);
                    }))
                    .child(div().text_color(rgb(CYAN)).child("C"))
                    .child("main.c"),
            )
            .child(
                div()
                    .px_4()
                    .py_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().text_color(rgb(ORANGE)).child("◆"))
                    .child("build.zig"),
            ),
        SidebarSection::Search => div()
            .child(
                div()
                    .px_3()
                    .py_2()
                    .text_size(px(11.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(MUTED))
                    .child("SEARCH"),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .text_size(px(12.))
                    .text_color(rgb(MUTED))
                    .child("Use the command field above to search files."),
            ),
        SidebarSection::SourceControl => div()
            .child(
                div()
                    .px_3()
                    .py_2()
                    .text_size(px(11.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(MUTED))
                    .child("SOURCE CONTROL"),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .text_size(px(12.))
                    .text_color(rgb(MUTED))
                    .child("No repository commit yet."),
            ),
        SidebarSection::Run => div()
            .child(
                div()
                    .px_3()
                    .py_2()
                    .text_size(px(11.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(MUTED))
                    .child("RUN"),
            )
            .child(
                div()
                    .id("sidebar-run")
                    .mx_3()
                    .mt_2()
                    .px_3()
                    .py_2()
                    .rounded_sm()
                    .bg(rgb(CYAN))
                    .text_color(rgb(BACKGROUND))
                    .font_weight(FontWeight::BOLD)
                    .cursor_pointer()
                    .hover(|style| style.opacity(0.88))
                    .on_click(context.listener(|app, _, _, context| app.run(context)))
                    .child("▶  Run main.c"),
            )
            .child(
                div()
                    .px_3()
                    .pt_3()
                    .text_size(px(12.))
                    .text_color(rgb(MUTED))
                    .child(format!("Completed runs: {}", app.activations)),
            ),
        SidebarSection::Extensions => div()
            .child(
                div()
                    .px_3()
                    .py_2()
                    .text_size(px(11.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(MUTED))
                    .child("TOOLCHAIN"),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .child("C / C++ language support")
                    .text_size(px(12.)),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .child("Zig 0.16.0 build core")
                    .text_size(px(12.))
                    .text_color(rgb(MUTED)),
            ),
    };

    div()
        .w(px(220.))
        .h_full()
        .flex_none()
        .bg(rgb(SURFACE))
        .border_r_1()
        .border_color(rgb(BORDER))
        .child(content)
}

fn welcome(context: &mut Context<LumaApp>) -> impl IntoElement {
    div()
        .id("welcome-scroll")
        .size_full()
        .overflow_y_scroll()
        .px_8()
        .py_8()
        .child(
            div()
                .w_full()
                .max_w(px(760.))
                .mx_auto()
                .flex()
                .flex_col()
                .gap_6()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(34.))
                                .font_weight(FontWeight::BOLD)
                                .child("Luma"),
                        )
                        .child(
                            div()
                                .text_size(px(16.))
                                .text_color(rgb(MUTED))
                                .child("打开，就能写 C。"),
                        )
                        .child(
                            div()
                                .mt_2()
                                .w(px(176.))
                                .px_3()
                                .py_1()
                                .rounded_sm()
                                .border_1()
                                .border_color(rgb(BORDER))
                                .bg(rgb(SURFACE))
                                .text_size(px(11.))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(div().size(px(7.)).rounded_full().bg(rgb(0xb6d65f)))
                                        .child("Lec · Learn. Code. Run."),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .gap_3()
                        .child(
                            div()
                                .id("quick-new-file")
                                .flex_1()
                                .min_h(px(118.))
                                .p_4()
                                .rounded_md()
                                .border_1()
                                .border_color(rgb(BORDER))
                                .bg(rgb(SURFACE))
                                .cursor_pointer()
                                .hover(|style| {
                                    style.bg(rgb(SURFACE_RAISED)).border_color(rgb(CYAN))
                                })
                                .on_click(context.listener(|app, _, window, context| {
                                    app.open_editor(
                                        "Created main.c · ready to edit",
                                        window,
                                        context,
                                    );
                                }))
                                .child(div().text_size(px(22.)).text_color(rgb(CYAN)).child("＋"))
                                .child(
                                    div()
                                        .mt_3()
                                        .font_weight(FontWeight::BOLD)
                                        .child("New C File"),
                                )
                                .child(
                                    div()
                                        .mt_1()
                                        .text_size(px(12.))
                                        .text_color(rgb(MUTED))
                                        .child("Start from a clean, runnable main.c"),
                                ),
                        )
                        .child(
                            div()
                                .id("quick-open-project")
                                .flex_1()
                                .min_h(px(118.))
                                .p_4()
                                .rounded_md()
                                .border_1()
                                .border_color(rgb(BORDER))
                                .bg(rgb(SURFACE))
                                .cursor_pointer()
                                .hover(|style| {
                                    style.bg(rgb(SURFACE_RAISED)).border_color(rgb(ORANGE))
                                })
                                .on_click(context.listener(|app, _, window, context| {
                                    app.open_editor(
                                        "Opened sample project · hello-c",
                                        window,
                                        context,
                                    );
                                }))
                                .child(div().text_size(px(21.)).text_color(rgb(ORANGE)).child("⌁"))
                                .child(
                                    div()
                                        .mt_3()
                                        .font_weight(FontWeight::BOLD)
                                        .child("Open Project"),
                                )
                                .child(
                                    div()
                                        .mt_1()
                                        .text_size(px(12.))
                                        .text_color(rgb(MUTED))
                                        .child("Resume the local hello-c workspace"),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(12.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(MUTED))
                                .child("RECENT PROJECTS"),
                        )
                        .child(
                            div()
                                .id("recent-hello-c")
                                .flex()
                                .items_center()
                                .justify_between()
                                .px_4()
                                .py_3()
                                .border_1()
                                .border_color(rgb(BORDER))
                                .bg(rgb(SURFACE))
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(SURFACE_RAISED)))
                                .on_click(context.listener(|app, _, window, context| {
                                    app.open_editor("Opened ~/Projects/hello-c", window, context);
                                }))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_3()
                                        .child(
                                            div()
                                                .text_color(rgb(CYAN))
                                                .font_weight(FontWeight::BOLD)
                                                .child("C"),
                                        )
                                        .child(
                                            div()
                                                .child(
                                                    div()
                                                        .font_weight(FontWeight::BOLD)
                                                        .child("hello-c"),
                                                )
                                                .child(
                                                    div()
                                                        .mt_1()
                                                        .text_size(px(11.))
                                                        .text_color(rgb(MUTED))
                                                        .child("~/Projects/hello-c"),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .text_size(px(11.))
                                        .child(
                                            div()
                                                .px_2()
                                                .py_1()
                                                .rounded_sm()
                                                .bg(rgb(0x15303a))
                                                .text_color(rgb(CYAN))
                                                .child("C17"),
                                        )
                                        .child(
                                            div()
                                                .px_2()
                                                .py_1()
                                                .rounded_sm()
                                                .bg(rgb(0x342719))
                                                .text_color(rgb(ORANGE))
                                                .child("Zig 0.16"),
                                        )
                                        .child(div().text_color(rgb(MUTED)).child("Today")),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(12.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(MUTED))
                                .child("PROJECT TEMPLATES"),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_3()
                                .child(
                                    div()
                                        .id("template-console")
                                        .flex_1()
                                        .px_4()
                                        .py_3()
                                        .border_1()
                                        .border_color(rgb(BORDER))
                                        .cursor_pointer()
                                        .hover(|style| style.bg(rgb(SURFACE_RAISED)))
                                        .on_click(context.listener(|app, _, window, context| {
                                            app.open_editor(
                                                "Created Console App template",
                                                window,
                                                context,
                                            );
                                        }))
                                        .child(
                                            div()
                                                .font_weight(FontWeight::BOLD)
                                                .child("Console App"),
                                        )
                                        .child(
                                            div()
                                                .mt_1()
                                                .text_size(px(11.))
                                                .text_color(rgb(MUTED))
                                                .child("main.c · build.zig"),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("template-single-file")
                                        .flex_1()
                                        .px_4()
                                        .py_3()
                                        .border_1()
                                        .border_color(rgb(BORDER))
                                        .cursor_pointer()
                                        .hover(|style| style.bg(rgb(SURFACE_RAISED)))
                                        .on_click(context.listener(|app, _, window, context| {
                                            app.open_editor(
                                                "Created Single File template",
                                                window,
                                                context,
                                            );
                                        }))
                                        .child(
                                            div()
                                                .font_weight(FontWeight::BOLD)
                                                .child("Single File"),
                                        )
                                        .child(
                                            div()
                                                .mt_1()
                                                .text_size(px(11.))
                                                .text_color(rgb(MUTED))
                                                .child("One source file, zero setup"),
                                        ),
                                ),
                        ),
                ),
        )
}

fn editor(app: &LumaApp, context: &mut Context<LumaApp>) -> impl IntoElement {
    let editor = app.editor.read(context);
    let dirty_marker = if editor.is_dirty() { " ●" } else { "" };
    let (line, column) = editor.cursor_position();

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(rgb(BACKGROUND))
        .child(
            div()
                .h(px(32.))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px_3()
                .border_b_1()
                .border_color(rgb(BORDER))
                .text_size(px(11.))
                .text_color(rgb(MUTED))
                .child(format!("hello-c / main.c{dirty_marker}"))
                .child("C17"),
        )
        .child(
            div()
                .flex_1()
                .min_h(px(0.))
                .overflow_hidden()
                .child(app.editor.clone()),
        )
        .child(
            div()
                .h(px(32.))
                .flex_none()
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .border_t_1()
                .border_color(rgb(BORDER))
                .text_size(px(11.))
                .child(
                    div()
                        .id("editor-run")
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(rgb(CYAN))
                        .text_color(rgb(BACKGROUND))
                        .font_weight(FontWeight::BOLD)
                        .cursor_pointer()
                        .hover(|style| style.opacity(0.88))
                        .on_click(context.listener(|app, _, _, context| app.run(context)))
                        .child("▶ Run"),
                )
                .child(div().text_color(rgb(MUTED)).child("Cmd/Ctrl+Enter"))
                .child(div().flex_1())
                .child(
                    div()
                        .text_color(rgb(MUTED))
                        .child(format!("Ln {line}, Col {column}")),
                ),
        )
}

fn learn_panel(context: &mut Context<LumaApp>) -> impl IntoElement {
    div()
        .w(px(286.))
        .h_full()
        .flex_none()
        .bg(rgb(SURFACE))
        .border_l_1()
        .border_color(rgb(BORDER))
        .p_4()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .text_size(px(12.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(MUTED))
                .child("LEARN"),
        )
        .child(
            div()
                .id("lesson-basics")
                .p_3()
                .border_1()
                .border_color(rgb(BORDER))
                .bg(rgb(BACKGROUND))
                .cursor_pointer()
                .hover(|style| style.border_color(rgb(CYAN)))
                .on_click(context.listener(|app, _, _, context| {
                    app.set_status("Lesson selected · C Basics", context);
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(div().font_weight(FontWeight::BOLD).child("C Basics"))
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(rgb(CYAN))
                                .child("8 min"),
                        ),
                )
                .child(
                    div()
                        .mt_2()
                        .text_size(px(12.))
                        .text_color(rgb(MUTED))
                        .child("Variables, functions, and your first program."),
                ),
        )
        .child(
            div()
                .id("lesson-compile")
                .p_3()
                .border_1()
                .border_color(rgb(BORDER))
                .bg(rgb(BACKGROUND))
                .cursor_pointer()
                .hover(|style| style.border_color(rgb(ORANGE)))
                .on_click(context.listener(|app, _, _, context| {
                    app.set_status("Lesson selected · Compile & Run", context);
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(div().font_weight(FontWeight::BOLD).child("Compile & Run"))
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(rgb(ORANGE))
                                .child("5 min"),
                        ),
                )
                .child(
                    div()
                        .mt_2()
                        .text_size(px(12.))
                        .text_color(rgb(MUTED))
                        .child("Understand the build, then run with one click."),
                ),
        )
        .child(
            div()
                .id("lesson-debug")
                .p_3()
                .border_1()
                .border_color(rgb(BORDER))
                .bg(rgb(BACKGROUND))
                .cursor_pointer()
                .hover(|style| style.border_color(rgb(CYAN)))
                .on_click(context.listener(|app, _, _, context| {
                    app.set_status("Lesson selected · Debug Loops", context);
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(div().font_weight(FontWeight::BOLD).child("Debug Loops"))
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(rgb(CYAN))
                                .child("6 min"),
                        ),
                )
                .child(
                    div()
                        .mt_2()
                        .text_size(px(12.))
                        .text_color(rgb(MUTED))
                        .child("Trace state and fix the off-by-one bug."),
                ),
        )
        .child(div().flex_1())
        .child(
            div()
                .pt_3()
                .border_t_1()
                .border_color(rgb(BORDER))
                .text_size(px(11.))
                .text_color(rgb(MUTED))
                .child("Built for beginners · Native GPUI"),
        )
}

pub(crate) fn render(app: &LumaApp, context: &mut Context<LumaApp>) -> impl IntoElement {
    let last_key = app
        .recent_keystrokes
        .last()
        .map(Keystroke::unparse)
        .unwrap_or_else(|| "—".to_owned());

    div()
        .id("luma-workbench")
        .role(Role::Application)
        .aria_label("Luma C and C++ IDE")
        .track_focus(&app.focus)
        .on_action(context.listener(LumaApp::focus_next))
        .on_action(context.listener(LumaApp::focus_previous))
        .size_full()
        .flex()
        .flex_col()
        .bg(rgb(BACKGROUND))
        .text_color(rgb(TEXT))
        .text_size(px(13.))
        .child(
            div()
                .h(px(42.))
                .flex_none()
                .flex()
                .items_center()
                .border_b_1()
                .border_color(rgb(BORDER))
                .bg(rgb(SURFACE))
                .child(
                    div()
                        .w(px(162.))
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_4()
                        .text_size(px(11.))
                        .text_color(rgb(MUTED))
                        .child("hello-c"),
                )
                .when(cfg!(target_os = "windows"), |header| {
                    header.child(
                        div()
                            .id("windows-run-command")
                            .role(Role::Button)
                            .aria_label("Run current file")
                            .aria_keyshortcuts("Control+Enter")
                            .focusable()
                            .tab_stop(true)
                            .px_3()
                            .py_1()
                            .rounded_sm()
                            .border_1()
                            .border_color(rgb(BORDER))
                            .focus_visible(|style| style.border_color(rgb(CYAN)))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(SURFACE_RAISED)))
                            .active(|style| style.opacity(0.75))
                            .on_click(|_, window, context| {
                                window.dispatch_action(Box::new(Activate), context);
                            })
                            .child("Run  Ctrl+Enter"),
                    )
                })
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_3()
                        .child(
                            div()
                                .size(px(22.))
                                .rounded_sm()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(rgb(CYAN))
                                .text_color(rgb(BACKGROUND))
                                .font_weight(FontWeight::BOLD)
                                .child("L"),
                        )
                        .child(div().font_weight(FontWeight::BOLD).child("Luma"))
                        .child(div().text_color(rgb(MUTED)).child("Learn. Code. Run.")),
                )
                .child(
                    div()
                        .w(px(314.))
                        .flex_none()
                        .pr_3()
                        .child(app.command_input.clone()),
                ),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .overflow_hidden()
                .child(
                    div()
                        .w(px(50.))
                        .h_full()
                        .flex_none()
                        .bg(rgb(0x0e141b))
                        .border_r_1()
                        .border_color(rgb(BORDER))
                        .flex()
                        .flex_col()
                        .items_center()
                        .py_2()
                        .gap_1()
                        .child(
                            div()
                                .id("rail-explorer")
                                .size(px(36.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .border_l_2()
                                .border_color(if app.active_sidebar == SidebarSection::Explorer {
                                    rgb(CYAN)
                                } else {
                                    rgb(0x0e141b)
                                })
                                .text_color(if app.active_sidebar == SidebarSection::Explorer {
                                    rgb(TEXT)
                                } else {
                                    rgb(MUTED)
                                })
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(SURFACE_RAISED)))
                                .on_click(context.listener(|app, _, _, context| {
                                    app.select_sidebar(
                                        SidebarSection::Explorer,
                                        "Explorer",
                                        context,
                                    );
                                }))
                                .child("▱"),
                        )
                        .child(
                            div()
                                .id("rail-search")
                                .size(px(36.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .border_l_2()
                                .border_color(if app.active_sidebar == SidebarSection::Search {
                                    rgb(CYAN)
                                } else {
                                    rgb(0x0e141b)
                                })
                                .text_color(if app.active_sidebar == SidebarSection::Search {
                                    rgb(TEXT)
                                } else {
                                    rgb(MUTED)
                                })
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(SURFACE_RAISED)))
                                .on_click(context.listener(|app, _, window, context| {
                                    app.select_search(window, context);
                                }))
                                .child("⌕"),
                        )
                        .child(
                            div()
                                .id("rail-source-control")
                                .size(px(36.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .border_l_2()
                                .border_color(
                                    if app.active_sidebar == SidebarSection::SourceControl {
                                        rgb(CYAN)
                                    } else {
                                        rgb(0x0e141b)
                                    },
                                )
                                .text_color(
                                    if app.active_sidebar == SidebarSection::SourceControl {
                                        rgb(TEXT)
                                    } else {
                                        rgb(MUTED)
                                    },
                                )
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(SURFACE_RAISED)))
                                .on_click(context.listener(|app, _, _, context| {
                                    app.select_sidebar(
                                        SidebarSection::SourceControl,
                                        "Source Control",
                                        context,
                                    );
                                }))
                                .child("⑂"),
                        )
                        .child(
                            div()
                                .id("rail-run")
                                .size(px(36.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .border_l_2()
                                .border_color(if app.active_sidebar == SidebarSection::Run {
                                    rgb(CYAN)
                                } else {
                                    rgb(0x0e141b)
                                })
                                .text_color(if app.active_sidebar == SidebarSection::Run {
                                    rgb(TEXT)
                                } else {
                                    rgb(MUTED)
                                })
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(SURFACE_RAISED)))
                                .on_click(context.listener(|app, _, _, context| {
                                    app.select_sidebar(
                                        SidebarSection::Run,
                                        "Run and Debug",
                                        context,
                                    );
                                }))
                                .child("▷"),
                        )
                        .child(
                            div()
                                .id("rail-extensions")
                                .size(px(36.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .border_l_2()
                                .border_color(if app.active_sidebar == SidebarSection::Extensions {
                                    rgb(CYAN)
                                } else {
                                    rgb(0x0e141b)
                                })
                                .text_color(if app.active_sidebar == SidebarSection::Extensions {
                                    rgb(TEXT)
                                } else {
                                    rgb(MUTED)
                                })
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(SURFACE_RAISED)))
                                .on_click(context.listener(|app, _, _, context| {
                                    app.select_sidebar(
                                        SidebarSection::Extensions,
                                        "Toolchain",
                                        context,
                                    );
                                }))
                                .child("◇"),
                        )
                        .child(div().flex_1())
                        .child(
                            div()
                                .id("rail-settings")
                                .size(px(36.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(rgb(MUTED))
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(SURFACE_RAISED)))
                                .on_click(context.listener(|app, _, _, context| {
                                    app.set_status("Settings · Stage 1", context);
                                }))
                                .child("⚙"),
                        ),
                )
                .child(sidebar(app, context))
                .child(
                    div()
                        .flex_1()
                        .h_full()
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        .child(
                            div()
                                .h(px(38.))
                                .flex_none()
                                .flex()
                                .items_end()
                                .bg(rgb(SURFACE))
                                .border_b_1()
                                .border_color(rgb(BORDER))
                                .child(
                                    div()
                                        .id("tab-welcome")
                                        .h_full()
                                        .px_4()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .border_b_2()
                                        .border_color(if app.active_tab == WorkspaceTab::Welcome {
                                            rgb(CYAN)
                                        } else {
                                            rgb(SURFACE)
                                        })
                                        .bg(if app.active_tab == WorkspaceTab::Welcome {
                                            rgb(BACKGROUND)
                                        } else {
                                            rgb(SURFACE)
                                        })
                                        .cursor_pointer()
                                        .on_click(context.listener(|app, _, _, context| {
                                            app.show_welcome("Welcome", context);
                                        }))
                                        .child(div().text_color(rgb(CYAN)).child("◈"))
                                        .child("Welcome"),
                                )
                                .when(app.editor_open, |tabs| {
                                    tabs.child(
                                        div()
                                            .id("tab-main-c")
                                            .h_full()
                                            .px_4()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .border_b_2()
                                            .border_color(
                                                if app.active_tab == WorkspaceTab::Editor {
                                                    rgb(ORANGE)
                                                } else {
                                                    rgb(SURFACE)
                                                },
                                            )
                                            .bg(if app.active_tab == WorkspaceTab::Editor {
                                                rgb(BACKGROUND)
                                            } else {
                                                rgb(SURFACE)
                                            })
                                            .cursor_pointer()
                                            .on_click(context.listener(
                                                |app, _, window, context| {
                                                    app.open_editor(
                                                        "Editing main.c",
                                                        window,
                                                        context,
                                                    );
                                                },
                                            ))
                                            .child(div().text_color(rgb(CYAN)).child("C"))
                                            .child("main.c"),
                                    )
                                }),
                        )
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .flex_1()
                                        .h_full()
                                        .when(app.active_tab == WorkspaceTab::Welcome, |body| {
                                            body.child(welcome(context))
                                        })
                                        .when(app.active_tab == WorkspaceTab::Editor, |body| {
                                            body.child(editor(app, context))
                                        }),
                                )
                                .child(learn_panel(context)),
                        ),
                ),
        )
        .child(
            div()
                .h(px(24.))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px_3()
                .border_t_1()
                .border_color(rgb(BORDER))
                .bg(rgb(0x0d151c))
                .text_size(px(11.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(div().text_color(rgb(CYAN)).child("⌁ main"))
                        .child("UTF-8")
                        .child("C17"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(div().text_color(rgb(0xb6d65f)).child(app.status.clone()))
                        .child(format!("Key: {last_key}")),
                ),
        )
}
