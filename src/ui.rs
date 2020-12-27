use iui::controls::{Button, Label, VerticalBox};
use iui::prelude::*;

pub fn show_dialog(message: &str) {
    eprintln!("{}", message);

    let ui_msg = message.split(": ").collect::<Vec<&str>>().join("\n");

    let ui = UI::init().expect("Failed to initialize UI.");
    let mut win = Window::new(&ui, "Bootstrap",
                              200, 100, WindowType::NoMenubar);

    win.modal_err(&ui, "Launcher Bootstrapper", &ui_msg);
    ui.quit();
}
