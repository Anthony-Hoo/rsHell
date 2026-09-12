//! Capture a native popover surface separately: window paintables omit GdkPopup surfaces.
use gtk::prelude::*;

pub(crate) fn capture(widget: &impl IsA<gtk::Widget>, name: &str) {
    let widget = widget.as_ref();
    assert!(widget.is_mapped() && widget.width() > 0 && widget.height() > 0);
    let font = widget.pango_context().font_description().unwrap();
    assert_eq!(
        font.size(),
        15 * gtk::pango::SCALE,
        "native overflow must inherit type-control, not ambient font sizing"
    );
    assert!(
        font.family().unwrap().contains("Segoe UI"),
        "native overflow must own font-ui"
    );
    let snapshot = gtk::Snapshot::new();
    gtk::WidgetPaintable::new(Some(widget)).snapshot(
        &snapshot,
        f64::from(widget.width()),
        f64::from(widget.height()),
    );
    let node = snapshot.to_node().expect("mapped native surface node");
    let renderer = gtk::gsk::CairoRenderer::new();
    renderer.realize(None).unwrap();
    let rect = gtk::graphene::Rect::new(0.0, 0.0, widget.width() as f32, widget.height() as f32);
    let texture = renderer.render_texture(&node, Some(&rect));
    renderer.unrealize();
    let stride = widget.width() as usize * 4;
    let mut pixels = vec![0u8; stride * widget.height() as usize];
    gtk::gdk::prelude::TextureExtManual::download(&texture, &mut pixels, stride);
    let sample = (widget.height() as usize / 2) * stride + 12 * 4;
    assert!(
        pixels[sample..sample + 3]
            .iter()
            .all(|channel| *channel < 128),
        "native overflow contents must not retain the ambient white surround: {:?}",
        &pixels[sample..sample + 4]
    );
    let Some(root) = std::env::var_os("RSHELL_FLUENT_SNAPSHOT_DIR") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    assert!(root.is_dir());
    let path = root.join(format!("{name}.png"));
    assert!(!path.exists(), "do not overwrite native surface evidence");
    texture.save_to_png(&path).unwrap();
    println!(
        "NATIVE_SURFACE path={} rect={rect:?} scale={} font={}",
        path.display(),
        widget.scale_factor(),
        widget.pango_context().font_description().unwrap()
    );
}
