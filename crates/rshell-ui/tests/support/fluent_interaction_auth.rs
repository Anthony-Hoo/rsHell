use super::*;
use rshell_core::{AppFailure, AppFailureCategory};

pub(super) fn run(mode: &str, width: i32, height: i32) {
    for kind in ["password", "passphrase", "keyboard"] {
        let c = Case::open(mode, width, height, kind, auth_request(kind), false);
        fill(&c);
        c.act("Submit");
        c.pending();
        let response = if kind == "keyboard" {
            Response::Answers
        } else {
            Response::Secret
        };
        c.port.verify(c.session, c.interaction, &[response]);
        c.shot("pending");
        c.main
            .emit(MainWindowMsg::AppEvent(AppEvent::OperationFailed(
                AppFailure::fatal(AppFailureCategory::Validation, layout::LONG_ERROR),
            )));
        actions::wait_error(&c.modal);
        layout::bottom(&c.modal);
        c.shot("long-error");
        let error = descendants(&c.modal)
            .into_iter()
            .find(|w| w.has_css_class("dialog-error"))
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap();
        layout::long_error(&c.modal, &error);
        layout::verify(c.main.widget().upcast_ref(), &c.modal);
        c.port.reject_next();
        fill(&c);
        c.act("Submit");
        wait_for_frame(&c.modal, "Busy rejected response", |modal| {
            descendants(modal)
                .into_iter()
                .filter_map(|w| w.downcast::<gtk::Label>().ok())
                .any(|l| l.has_css_class("dialog-error") && l.text().contains("busy"))
        });
        c.port
            .verify(c.session, c.interaction, &[response, response]);
        assert!(button(&c.modal, "Submit").is_sensitive());
        c.shot("busy-rejected");
        fill(&c);
        c.act("Submit");
        c.pending();
        c.port
            .verify(c.session, c.interaction, &[response, response, response]);
        c.shot("retry-pending");
        c.ack();
    }
    for (name, action, ordinary_first) in [
        ("auth-cancel", "Cancel", false),
        ("auth-escape-adversarial", "Escape", true),
    ] {
        let c = Case::open(
            mode,
            width,
            height,
            name,
            auth_request("password"),
            ordinary_first,
        );
        fill(&c);
        c.act(action);
        c.pending();
        c.port.verify(c.session, c.interaction, &[Response::Cancel]);
        c.shot("cancel-awaiting-ack");
        c.ack();
    }
}

fn fill(c: &Case) {
    for widget in descendants(&c.modal) {
        if let Ok(entry) = widget.clone().downcast::<gtk::Entry>() {
            assert!(entry.is_sensitive());
            entry.set_text("synthetic-visible-answer");
        }
        if let Ok(entry) = widget.downcast::<gtk::PasswordEntry>() {
            assert!(entry.is_sensitive());
            assert!(!entry.shows_peek_icon());
            entry.set_text("synthetic-disposable-answer");
        }
    }
    wait_for_frame(&c.modal, "native answers delivered", |_| true);
}
