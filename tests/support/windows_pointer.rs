//! Real Windows pointer input; restore our cursor/button state on success or panic.
use super::frames::wait_for_frame;
use relm4::gtk::{self, prelude::*};
use std::{cell::Cell, ffi::c_void, rc::Rc};

pub(super) fn drag(window: &gtk::ApplicationWindow, canvas: &gtk::Widget) {
    let controllers = canvas.observe_controllers();
    let click = (0..controllers.n_items())
        .filter_map(|i| controllers.item(i))
        .find_map(|c| c.downcast::<gtk::GestureClick>().ok())
        .unwrap();
    let pressed = Rc::new(Cell::new(false));
    let released = Rc::new(Cell::new(false));
    let seen = pressed.clone();
    let press = click.connect_pressed(move |g, _, x, y| {
        assert_eq!(g.current_button(), 1);
        assert!((x - 30.0).abs() <= 1.0 && (y - 10.0).abs() <= 1.0);
        seen.set(true);
    });
    let seen = released.clone();
    let release = click.connect_released(move |g, _, x, y| {
        assert_eq!(g.current_button(), 1);
        assert!((x - 130.0).abs() <= 1.0 && (y - 10.0).abs() <= 1.0);
        seen.set(true);
    });
    let bounds = canvas.compute_bounds(window).unwrap();
    let (surface_x, surface_y) = window.surface_transform();
    let scale = window.scale_factor() as f32;
    let point = |x: f32| {
        (
            ((bounds.x() + x + surface_x as f32) * scale) as i32,
            ((bounds.y() + 10.0 + surface_y as f32) * scale) as i32,
        )
    };
    let hwnd = unsafe { gdk_win32_surface_get_handle(window.surface().unwrap().as_ptr()) };
    assert!(!hwnd.is_null());
    let mut original = Point { x: 0, y: 0 };
    assert_ne!(unsafe { GetCursorPos(&mut original) }, 0);
    let cursor = CursorRestore(original);
    assert_ne!(unsafe { SetForegroundWindow(hwnd) }, 0);
    // Stay outside GtkPaned's wider native separator hit region.
    move_pointer(hwnd, point(30.0));
    wait_for_frame(canvas, "pointer positioning painted", |_| true);
    unsafe {
        mouse_event(0x0002, 0, 0, 0, 0);
    }
    wait_for_frame(canvas, "actual native left-button press", move |_| {
        pressed.get()
    });
    move_pointer(hwnd, point(130.0));
    wait_for_frame(canvas, "native drag movement", |_| true);
    unsafe {
        mouse_event(0x0004, 0, 0, 0, 0);
    }
    wait_for_frame(canvas, "actual native left-button release", move |_| {
        released.get()
    });
    click.disconnect(press);
    click.disconnect(release);
    drop(cursor);
    println!(
        "POINTER_INPUT native_press_release=true canvas_local=30,10..130,10 surface_transform={surface_x},{surface_y} cursor_restored=true"
    );
}

fn move_pointer(hwnd: *mut c_void, (x, y): (i32, i32)) {
    let mut point = Point { x, y };
    assert_ne!(unsafe { ClientToScreen(hwnd, &mut point) }, 0);
    assert_ne!(unsafe { SetCursorPos(point.x, point.y) }, 0);
}

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}
struct CursorRestore(Point);
impl Drop for CursorRestore {
    fn drop(&mut self) {
        unsafe {
            mouse_event(0x0004, 0, 0, 0, 0);
            SetCursorPos(self.0.x, self.0.y);
        }
    }
}

#[link(name = "gtk-4")]
unsafe extern "C" {
    fn gdk_win32_surface_get_handle(surface: *mut gtk::gdk::ffi::GdkSurface) -> *mut c_void;
}
#[link(name = "user32")]
unsafe extern "system" {
    fn GetCursorPos(point: *mut Point) -> i32;
    fn SetCursorPos(x: i32, y: i32) -> i32;
    fn ClientToScreen(hwnd: *mut c_void, point: *mut Point) -> i32;
    fn SetForegroundWindow(hwnd: *mut c_void) -> i32;
    fn mouse_event(flags: u32, dx: u32, dy: u32, data: u32, extra: usize);
}
