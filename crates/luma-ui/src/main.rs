#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod app;
mod editor;
mod text_input;
mod text_offset;
mod view;

use app::LumaApp;
use gpui::{
    App, AppContext, Bounds, KeyBinding, Menu, MenuItem, SystemMenuType, WindowBounds,
    WindowOptions, actions, px, size,
};
use gpui_platform::application;

actions!(luma, [Activate, FocusNext, FocusPrevious, Quit]);

fn quit(_: &Quit, context: &mut App) {
    context.quit();
}

fn main() {
    #[cfg(target_os = "macos")]
    // GPUIWindow is registered before main and remains loaded for the process lifetime.
    unsafe {
        accesskit_macos::add_focus_forwarder_to_window_class("GPUIWindow");
    }

    application().run(|context: &mut App| {
        context.on_action(quit);
        editor::register_key_bindings(context);
        text_input::register_key_bindings(context);
        context.bind_keys([
            KeyBinding::new("tab", FocusNext, None),
            KeyBinding::new("shift-tab", FocusPrevious, None),
            KeyBinding::new("cmd-enter", Activate, None),
            KeyBinding::new("ctrl-enter", Activate, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("ctrl-q", Quit, None),
        ]);
        context.set_menus([Menu::new("Luma").items([
            MenuItem::action("Run", Activate),
            MenuItem::separator(),
            MenuItem::os_submenu("Services", SystemMenuType::Services),
            MenuItem::separator(),
            MenuItem::action("Quit Luma", Quit),
        ])]);

        let bounds = Bounds::centered(None, size(px(1180.), px(760.)), context);
        let window = context
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("Luma — Learn. Code. Run.".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, context| context.new(|context| LumaApp::new(window, context)),
            )
            .expect("failed to open Luma window");
        let app = window
            .update(context, |_, _, context| context.entity())
            .expect("failed to access Luma window");
        context.on_action({
            let app = app.clone();
            move |_: &Activate, context| {
                app.update(context, |app, context| app.run(context));
            }
        });
        context
            .observe_keystrokes(move |event, _, context| {
                app.update(context, |app, context| {
                    app.record_keystroke(event.keystroke.clone(), context);
                });
            })
            .detach();
        context.activate(true);
    });
}
