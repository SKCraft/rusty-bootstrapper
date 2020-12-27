use iui::controls::{Button, Label, VerticalBox};
use iui::prelude::*;

pub fn show_dialog(message: &str) {
    eprintln!("{}", message);

    let ui_msg = message.split(": ").collect::<Vec<&str>>().join("\n");

    let ui = UI::init().unwrap();
    let mut win = Window::new(&ui, "Launcher Bootstrapper",
                              200, 100, WindowType::NoMenubar);

    let mut vbox = VerticalBox::new(&ui);
    vbox.set_padded(&ui, true);

    let label = Label::new(&ui, &ui_msg);
    let mut button = Button::new(&ui, "OK");
    button.on_clicked(&ui, {
        let ui = ui.clone();
        move |_| {
            ui.quit();
        }
    });

    vbox.append(&ui, label, LayoutStrategy::Stretchy);
    vbox.append(&ui, button, LayoutStrategy::Compact);

    win.set_child(&ui, vbox);
    win.show(&ui);

    ui.main();
}
