//! Shared bounded GTK frame pump; no dependency on a particular test binary.
use relm4::gtk::{self, prelude::*};
use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

pub(crate) fn iterate_until(deadline: Instant, mut ready: impl FnMut() -> bool) -> bool {
    let context = gtk::glib::MainContext::default();
    loop {
        if Instant::now() >= deadline {
            return false;
        }
        let completed = ready();
        if Instant::now() >= deadline {
            return false;
        }
        if completed {
            return true;
        }
        // One nonblocking dispatch: continuously-ready sources cannot defeat the deadline.
        if !context.iteration(false) {
            std::thread::yield_now();
        }
    }
}

pub(crate) fn wait_for_frame(
    widget: &impl IsA<gtk::Widget>,
    description: &str,
    ready: impl Fn(&gtk::Widget) -> bool + 'static,
) {
    let deadline = Instant::now() + Duration::from_secs(2);
    let widget = widget.as_ref();
    assert!(
        iterate_until(deadline, || widget.frame_clock().is_some()),
        "{description}: frame clock unavailable before deadline"
    );
    let clock = widget.frame_clock().unwrap();
    let painted = Rc::new(Cell::new(false));
    let signal_painted = painted.clone();
    let target = widget.clone();
    let signal = clock.connect_after_paint(move |_| {
        if target.is_mapped() && target.width() > 0 && target.height() > 0 && ready(&target) {
            signal_painted.set(true);
        }
    });
    widget.queue_draw();
    clock.request_phase(gtk::gdk::FrameClockPhase::PAINT | gtk::gdk::FrameClockPhase::AFTER_PAINT);
    let completed = iterate_until(deadline, || painted.get());
    clock.disconnect(signal);
    assert!(completed, "{description}: ready frame missed deadline");
}
