//! Reconcile pre-allocation initial focus once, using the allocated outer control.
use gtk::prelude::*;
use relm4::gtk;
use std::{cell::RefCell, rc::Rc};

pub(super) fn reveal_after_paint(input: &gtk::Widget, clock: &gtk::gdk::FrameClock) {
    let input = input.downgrade();
    let handler = Rc::new(RefCell::new(None));
    let completed = handler.clone();
    *handler.borrow_mut() = Some(clock.connect_after_paint(move |clock| {
        // Disconnect even if the interaction closed or focus moved before paint.
        if let Some(handler) = completed.borrow_mut().take() {
            clock.disconnect(handler);
        }
        let Some(input) = input
            .upgrade()
            .filter(|w| w.is_mapped() && w.is_sensitive())
        else {
            return;
        };
        let focused = input
            .root()
            .and_then(|root| gtk::prelude::RootExt::focus(&root));
        if !focused.is_some_and(|w| w == input || w.is_ancestor(&input)) {
            return;
        }
        let Some(scroll) = input
            .ancestor(gtk::ScrolledWindow::static_type())
            .and_then(|w| w.downcast::<gtk::ScrolledWindow>().ok())
        else {
            return; // Host-key actions are in the fixed footer, not the body.
        };
        let Some(viewport) = scroll.child() else {
            return;
        };
        let Some(bounds) = input.compute_bounds(&viewport) else {
            return;
        };
        let adjustment = scroll.vadjustment();
        let top = adjustment.value() + f64::from(bounds.y());
        adjustment.clamp_page(top, top + f64::from(bounds.height()));
    }));
}
