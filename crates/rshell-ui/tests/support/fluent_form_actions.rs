use super::{fluent_native::*, layout};
use gtk::prelude::*;

pub(crate) fn trigger(root: &gtk::Widget, tooltip: &str) -> gtk::Button {
    descendants(root)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Button>().ok())
        .find(|w| w.is_mapped() && w.tooltip_text().as_deref() == Some(tooltip))
        .unwrap()
}

pub(crate) fn open(
    root: &gtk::Widget,
    tooltip: &str,
    class: &'static str,
) -> (gtk::Widget, gtk::Button) {
    let trigger = trigger(root, tooltip);
    assert!(trigger.grab_focus());
    wait_for_frame(&trigger, "trigger focused before open", focus_within);
    trigger.emit_clicked();
    wait_for_frame(root, "intentional modal focus per open", move |w| {
        modal_ready(w, class)
    });
    let modal = layout::modal(root, class);
    assert!(focus_within(&modal));
    println!("FLUENT_MODAL_OPEN class={class} root_focus_first=true");
    (modal, trigger)
}

pub(crate) fn closed(root: &gtk::Widget, modal: &gtk::Widget, trigger: &gtk::Button) {
    let modal = modal.clone();
    let expected = trigger.clone().upcast::<gtk::Widget>();
    wait_for_frame(root, "precise trigger restoration", move |w| {
        !modal.is_mapped()
            && w.root()
                .and_then(|r| gtk::prelude::RootExt::focus(&r))
                .as_ref()
                == Some(&expected)
    });
    println!("FLUENT_MODAL_CLOSE exact_trigger_restored=true");
}

pub(crate) fn escape(root: &gtk::Widget, modal: &gtk::Widget, trigger: &gtk::Button) {
    native_key(
        modal,
        gtk::gdk::Key::Escape,
        gtk::gdk::ModifierType::empty(),
    );
    closed(root, modal, trigger);
}

pub(super) fn fill(field: &gtk::Entry, value: &'static str) {
    // Match two native user edits (delete then insert), rather than GtkEditable's
    // synchronous replacement pair racing the queued Settings renderer.
    if !field.text().is_empty() {
        field.set_text("");
        wait_for_frame(field, "native deletion", |w| {
            w.clone()
                .downcast::<gtk::Entry>()
                .unwrap()
                .text()
                .is_empty()
        });
    }
    if !value.is_empty() {
        field.set_text(value);
        wait_for_frame(field, "native insertion", move |w| {
            w.clone().downcast::<gtk::Entry>().unwrap().text() == value
        });
    }
}

pub(super) fn entry(root: &gtk::Widget, name: &str) -> gtk::Entry {
    field_for_label(root, name).downcast().unwrap()
}

pub(super) fn error(root: &gtk::Widget) -> gtk::Label {
    descendants(root)
        .into_iter()
        .find(|w| w.has_css_class("dialog-error") && w.is_visible())
        .unwrap()
        .downcast()
        .unwrap()
}

pub(crate) fn wait_error(root: &gtk::Widget) {
    wait_for_frame(root, "visible production validation", |w| {
        descendants(w)
            .iter()
            .any(|w| w.has_css_class("dialog-error") && w.is_visible())
    });
}

pub(super) fn state(window: &gtk::ApplicationWindow, modal: &gtk::Widget, mode: &str, state: &str) {
    layout::verify(window.upcast_ref(), modal);
    capture(window, mode, state);
    println!("FLUENT_STATE mode={mode} state={state} layout=true");
}
