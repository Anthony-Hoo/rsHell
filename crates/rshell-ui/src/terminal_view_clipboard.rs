use gtk::gdk;
use relm4::ComponentSender;

use crate::{TerminalView, TerminalViewError, TerminalViewMsg, TerminalViewOutput};

pub(crate) fn read(clipboard: &gdk::Clipboard, sender: &ComponentSender<TerminalView>) {
    let input = sender.input_sender().clone();
    let sender = sender.clone();
    clipboard.read_text_async(gtk::gio::Cancellable::NONE, move |result| {
        match map_clipboard_read_result(result) {
            Ok(Some(text)) => {
                let _ = input.send(TerminalViewMsg::PasteText(text));
            }
            Ok(None) => {}
            Err(error) => {
                let _ = sender.output(TerminalViewOutput::Error(error));
            }
        }
    });
}

pub(crate) fn map_clipboard_read_result<T: Into<String>, E>(
    result: Result<Option<T>, E>,
) -> Result<Option<String>, TerminalViewError> {
    result
        .map(|text| text.map(Into::into))
        .map_err(|_| TerminalViewError::ClipboardUnavailable)
}
