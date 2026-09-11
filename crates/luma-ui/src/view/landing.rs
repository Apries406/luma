use crate::app::LumaApp;
use gpui::{Context, FontWeight, IntoElement, Role, div, prelude::*, px, rgb};

pub(crate) fn render(context: &mut Context<LumaApp>) -> impl IntoElement {
    let ink = rgb(0x272b32);
    let paper = rgb(0xf3efe7);
    let cyan = rgb(0x25c6d8);
    let orange = rgb(0xef9b47);

    div()
        .id("luma-landing")
        .relative()
        .size_full()
        .overflow_hidden()
        .bg(paper)
        .text_color(ink)
        .child(
            div()
                .absolute()
                .inset_0()
                .opacity(0.28)
                .font_family("Menlo")
                .text_size(px(14.))
                .text_color(rgb(0x59616b))
                .child(
                    div()
                        .absolute()
                        .top(px(104.))
                        .left(px(52.))
                        .w(px(470.))
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child("#include <stdio.h>")
                        .child("int main(void) {")
                        .child(div().pl_5().child("printf(\"Hello, Luma!\\n\");"))
                        .child(div().pl_5().text_color(cyan).child("return 0;"))
                        .child("}"),
                )
                .child(
                    div()
                        .absolute()
                        .top(px(214.))
                        .right(px(38.))
                        .w(px(420.))
                        .h(px(260.))
                        .border_1()
                        .border_color(rgb(0xaab1b7))
                        .child(
                            div()
                                .h(px(32.))
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_3()
                                .border_b_1()
                                .border_color(rgb(0xaab1b7))
                                .child(div().size(px(7.)).rounded_full().bg(rgb(0xe46a5d)))
                                .child(div().size(px(7.)).rounded_full().bg(orange))
                                .child(div().size(px(7.)).rounded_full().bg(rgb(0x74aa62))),
                        )
                        .child(
                            div()
                                .p_4()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child("01  #include <stdio.h>")
                                .child("02")
                                .child(div().text_color(cyan).child("03  int main(void) {"))
                                .child(div().pl_5().child("04  printf(\"Learn. Code. Run.\");"))
                                .child(div().pl_5().text_color(orange).child("05  return 0;"))
                                .child("06  }"),
                        ),
                )
                .child(
                    div()
                        .absolute()
                        .bottom(px(90.))
                        .left(px(74.))
                        .w(px(420.))
                        .h(px(92.))
                        .border_1()
                        .border_color(rgb(0xaab1b7))
                        .p_4()
                        .text_size(px(12.))
                        .child(div().text_color(cyan).child("✓ Build succeeded"))
                        .child(div().mt_2().child("$ ./hello"))
                        .child(div().text_color(orange).child("Hello, Luma!")),
                )
                .child(
                    div()
                        .absolute()
                        .bottom(px(56.))
                        .right(px(130.))
                        .w(px(240.))
                        .flex()
                        .flex_col()
                        .gap_2()
                        .text_size(px(12.))
                        .child("◇  main.c")
                        .child("⌁  build.zig")
                        .child(div().text_color(cyan).child("●  zero diagnostics")),
                ),
        )
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .h(px(68.))
                .flex()
                .items_center()
                .justify_between()
                .px_8()
                .border_b_1()
                .border_color(rgb(0xd8d1c5))
                .bg(paper.opacity(0.94))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .size(px(31.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(ink)
                                .text_color(paper)
                                .font_weight(FontWeight::BOLD)
                                .child("L"),
                        )
                        .child(
                            div()
                                .font_family("Georgia")
                                .text_size(px(21.))
                                .font_weight(FontWeight::BOLD)
                                .child("Luma"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_6()
                        .text_size(px(12.))
                        .font_weight(FontWeight::BOLD)
                        .child(div().text_color(cyan).child("Home"))
                        .child(
                            div()
                                .id("landing-docs")
                                .cursor_pointer()
                                .hover(|style| style.text_color(cyan))
                                .on_click(context.listener(|app, _, _, context| {
                                    app.show_welcome("Workspace guide", context);
                                }))
                                .child("Docs"),
                        )
                        .child(
                            div()
                                .id("landing-gpui")
                                .cursor_pointer()
                                .hover(|style| style.text_color(cyan))
                                .on_click(|_, _, context| {
                                    context.open_url("https://github.com/zed-industries/zed");
                                })
                                .child("GPUI"),
                        ),
                ),
        )
        .child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .w(px(720.))
                        .px_8()
                        .py_8()
                        .bg(paper.opacity(0.9))
                        .shadow_lg()
                        .flex()
                        .flex_col()
                        .items_center()
                        .text_center()
                        .child(
                            div()
                                .text_size(px(12.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(orange)
                                .child("A native C / C++ workspace for beginners"),
                        )
                        .child(
                            div()
                                .mt_3()
                                .font_family("Georgia")
                                .text_size(px(54.))
                                .font_weight(FontWeight::BOLD)
                                .child("Open. Write C."),
                        )
                        .child(
                            div()
                                .font_family("Georgia")
                                .text_size(px(54.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(cyan)
                                .child("That’s it."),
                        )
                        .child(
                            div()
                                .mt_5()
                                .max_w(px(540.))
                                .text_size(px(16.))
                                .line_height(px(25.))
                                .text_color(rgb(0x5f6670))
                                .child(
                                    "A calm place to learn, code, and run C — with the toolchain ready when you are.",
                                ),
                        )
                        .child(
                            div()
                                .mt_6()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .id("landing-start")
                                        .role(Role::Button)
                                        .aria_label("Start coding")
                                        .tab_stop(true)
                                        .focusable()
                                        .px_5()
                                        .py_3()
                                        .bg(ink)
                                        .text_color(paper)
                                        .font_weight(FontWeight::BOLD)
                                        .cursor_pointer()
                                        .hover(|style| style.bg(rgb(0x3d424b)))
                                        .active(|style| style.opacity(0.86))
                                        .on_click(context.listener(|app, _, window, context| {
                                            app.open_editor(
                                                "Created main.c · ready to edit",
                                                window,
                                                context,
                                            );
                                        }))
                                        .child("Start Coding  →"),
                                )
                                .child(
                                    div()
                                        .id("landing-explore")
                                        .role(Role::Button)
                                        .aria_label("Explore workspace")
                                        .tab_stop(true)
                                        .focusable()
                                        .px_5()
                                        .py_3()
                                        .border_1()
                                        .border_color(rgb(0xa7a097))
                                        .font_weight(FontWeight::BOLD)
                                        .cursor_pointer()
                                        .hover(|style| style.border_color(cyan).text_color(cyan))
                                        .on_click(context.listener(|app, _, _, context| {
                                            app.show_welcome("Welcome to Luma", context);
                                        }))
                                        .child("Explore Workspace"),
                                ),
                        )
                        .child(
                            div()
                                .mt_5()
                                .font_family("Georgia")
                                .text_size(px(12.))
                                .text_color(rgb(0x7d756b))
                                .child("Built by Lec · Powered by Rust, GPUI, and Zig 0.16"),
                        ),
                ),
        )
}
