use native_dialog::{MessageDialog, MessageType};

pub fn show_dialog(message: &str) {
    eprintln!("{}", message);

    let ui_msg = message.split(": ").collect::<Vec<&str>>().join(":\n");
    let maybe_err = MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title("Launcher Bootstrapper")
        .set_text(&ui_msg)
        .show_alert()
        .err();

    if let Some(err) = maybe_err {
        eprintln!("Failed to show dialog: {:?}", err);
    }
}
