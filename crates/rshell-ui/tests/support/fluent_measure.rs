//! Allocation evidence contains only widget types/classes and ordinal indices, never input text.
use super::fluent_native::descendants;
use gtk::prelude::*;

pub(crate) fn record(root: &gtk::Widget, mode: &str, class: &str) {
    let modal = descendants(root)
        .into_iter()
        .find(|w| w.has_css_class(class) && w.is_mapped())
        .unwrap();
    let bounds = modal.compute_bounds(root).unwrap();
    println!(
        "FLUENT_MODAL mode={mode} surface={class} bounds={},{},{},{} gap={}",
        bounds.x(),
        bounds.y(),
        bounds.width(),
        bounds.height(),
        modal.clone().downcast::<gtk::Box>().unwrap().spacing()
    );
    for (index, widget) in descendants(&modal)
        .into_iter()
        .filter(|w| w.is_mapped())
        .enumerate()
    {
        let part = [
            "dialog-header",
            "dialog-body",
            "dialog-footer",
            "dialog-section",
            "dialog-instruction",
        ]
        .into_iter()
        .find(|c| widget.has_css_class(c));
        let control = widget.is::<gtk::Entry>()
            || widget.is::<gtk::PasswordEntry>()
            || widget.is::<gtk::SpinButton>()
            || widget.is::<gtk::DropDown>()
            || widget.is::<gtk::Button>();
        if control || part.is_some() {
            let b = widget.compute_bounds(&modal).unwrap();
            let (_, natural, _, _) =
                widget.measure(gtk::Orientation::Vertical, b.width().ceil() as i32);
            let font = widget.pango_context().font_description().unwrap();
            let family = widget
                .pango_context()
                .load_font(&font)
                .map(|font| font.describe().to_string());
            println!(
                "FLUENT_MEASURE mode={mode} surface={class} ordinal={index} type={} part={} bounds={},{},{},{} natural={natural} font={font} resolved={family:?}",
                widget.type_().name(),
                part.unwrap_or("control"),
                b.x(),
                b.y(),
                b.width(),
                b.height()
            );
        }
        if let Ok(grid) = widget.downcast::<gtk::Grid>() {
            println!(
                "FLUENT_GRID mode={mode} surface={class} ordinal={index} rowgap={} colgap={}",
                grid.row_spacing(),
                grid.column_spacing()
            );
        }
    }
}
