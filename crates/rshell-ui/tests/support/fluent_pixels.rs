use relm4::gtk::{self, gdk::prelude::TextureExtManual, prelude::*};
use rshell_ui::{NativeByteOrder, argb32_native_to_rgba};
use std::path::PathBuf;

fn texture(widget: &gtk::Widget) -> gtk::gdk::Texture {
    let paintable = gtk::WidgetPaintable::new(Some(widget));
    let snapshot = gtk::Snapshot::new();
    paintable.snapshot(
        &snapshot,
        f64::from(widget.width()),
        f64::from(widget.height()),
    );
    let node = snapshot
        .to_node()
        .expect("mapped GTK widget must have a render node");
    let renderer = gtk::gsk::CairoRenderer::new();
    renderer.realize(None).unwrap();
    let viewport =
        gtk::graphene::Rect::new(0.0, 0.0, widget.width() as f32, widget.height() as f32);
    let texture = renderer.render_texture(&node, Some(&viewport));
    renderer.unrealize();
    texture
}

pub(crate) fn capture(window: &gtk::ApplicationWindow, mode: &str, state: &str) {
    if let Some(directory) = std::env::var_os("RSHELL_FLUENT_SNAPSHOT_DIR") {
        let directory = PathBuf::from(directory);
        assert!(
            directory.is_dir(),
            "snapshot destination must be registered and created by the caller"
        );
        let path = directory.join(format!("{mode}-{state}.png"));
        assert!(!path.exists(), "never overwrite previous native evidence");
        texture(window.upcast_ref()).save_to_png(&path).unwrap();
        println!(
            "FLUENT_SNAPSHOT {} realized={}x{}",
            path.display(),
            window.width(),
            window.height()
        );
    }
}

pub(crate) struct Pixels {
    data: Vec<u8>,
    width: i32,
    pub(crate) height: i32,
}
impl Pixels {
    pub(crate) fn at(&self, x: i32, y: i32) -> [u8; 4] {
        let index = ((y * self.width + x) * 4) as usize;
        self.data[index..index + 4].try_into().unwrap()
    }
    pub(crate) fn maximum_text_contrast(&self) -> f64 {
        let luminance = |value: f64| {
            let channel = value / 255.0;
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        };
        self.data
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| {
                let alpha = f64::from(pixel[3]) / 255.0;
                let rendered =
                    |channel| luminance(f64::from(pixel[channel]) * alpha + 43.0 * (1.0 - alpha));
                (0.2126 * rendered(0) + 0.7152 * rendered(1) + 0.0722 * rendered(2) + 0.05)
                    / (luminance(43.0) + 0.05)
            })
            .fold(1.0, f64::max)
    }
}

pub(crate) fn pixels(widget: &gtk::Widget) -> Pixels {
    let texture = texture(widget);
    let (width, height) = (texture.width(), texture.height());
    let mut native = vec![0; (width * height * 4) as usize];
    texture.download(&mut native, (width * 4) as usize);
    Pixels {
        data: argb32_native_to_rgba(&native, NativeByteOrder::current()).unwrap(),
        width,
        height,
    }
}

pub(crate) fn is_control(pixel: [u8; 4]) -> bool {
    pixel[..3]
        .iter()
        .all(|channel| (i16::from(*channel) - 43).abs() <= 2)
        && pixel[3] == 255
}

pub(crate) fn is_accent(pixel: [u8; 4]) -> bool {
    pixel[0] < 140 && pixel[1] > 170 && pixel[2] > 210 && pixel[3] > 240
}
