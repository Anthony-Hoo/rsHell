use gtk::prelude::*;
use rshell_core::{PaneTree, SplitAxis};

pub(crate) fn assert_grid_tree(tree: &PaneTree) {
    let PaneTree::Split {
        axis,
        first,
        second,
        ..
    } = tree
    else {
        panic!("Grid requires a split root");
    };
    for child in [first, second] {
        let PaneTree::Split {
            axis: child_axis,
            first,
            second,
            ..
        } = child.as_ref()
        else {
            panic!("Grid requires two split halves, not a comb");
        };
        assert_ne!(axis, child_axis, "Grid halves must be orthogonal to root");
        assert!(matches!(first.as_ref(), PaneTree::Leaf { .. }));
        assert!(matches!(second.as_ref(), PaneTree::Leaf { .. }));
    }
}

pub(crate) fn assert_allocations(host: &gtk::Widget, tree: &PaneTree, grid: bool) {
    let overlay = host
        .first_child()
        .unwrap()
        .downcast::<gtk::Overlay>()
        .unwrap();
    let root = overlay.child().expect("rendered pane root");
    assert_orientation(&root, tree);
    let mut surfaces = Vec::new();
    leaves(&root, &mut surfaces);
    assert_eq!(surfaces.len(), tree.pane_ids().len());
    let rects = surfaces
        .iter()
        .map(|pane| {
            assert!(pane.is_mapped());
            let b = pane.compute_bounds(host).unwrap();
            assert!(b.width() > 0.0 && b.height() > 0.0, "positive pane {b:?}");
            assert!(
                b.x() >= -1.0
                    && b.y() >= -1.0
                    && b.x() + b.width() <= host.width() as f32 + 1.0
                    && b.y() + b.height() <= host.height() as f32 + 1.0,
                "contained pane {b:?}"
            );
            println!(
                "PANE_RECT x={} y={} width={} height={}",
                b.x(),
                b.y(),
                b.width(),
                b.height()
            );
            b
        })
        .collect::<Vec<_>>();
    for (i, a) in rects.iter().enumerate() {
        for b in &rects[i + 1..] {
            assert!(
                a.x() + a.width() <= b.x() + 1.0
                    || b.x() + b.width() <= a.x() + 1.0
                    || a.y() + a.height() <= b.y() + 1.0
                    || b.y() + b.height() <= a.y() + 1.0,
                "panes overlap: {a:?} {b:?}"
            );
        }
    }
    if !grid {
        return;
    }
    assert_grid_tree(tree);
    let mut quadrants = [None; 4];
    for b in rects {
        let right = b.x() + b.width() / 2.0 > host.width() as f32 / 2.0;
        let bottom = b.y() + b.height() / 2.0 > host.height() as f32 / 2.0;
        let slot = usize::from(bottom) * 2 + usize::from(right);
        assert!(
            quadrants[slot].replace(b).is_none(),
            "duplicate quadrant {slot}"
        );
    }
    let [tl, tr, bl, br] = quadrants.map(|r| r.expect("one pane per quadrant"));
    for (a, b) in [(tl, tr), (bl, br)] {
        assert!(
            (a.y() - b.y()).abs() <= 2.0 && (a.height() - b.height()).abs() <= 2.0,
            "row alignment: {a:?} {b:?}"
        );
    }
    for (a, b) in [(tl, bl), (tr, br)] {
        assert!(
            (a.x() - b.x()).abs() <= 2.0 && (a.width() - b.width()).abs() <= 2.0,
            "column alignment: {a:?} {b:?}"
        );
    }
}

fn assert_orientation(widget: &gtk::Widget, tree: &PaneTree) {
    match tree {
        PaneTree::Leaf { .. } => assert!(widget.has_css_class("pane-surface")),
        PaneTree::Split {
            axis,
            first,
            second,
            ..
        } => {
            let split = widget
                .clone()
                .downcast::<gtk::Paned>()
                .expect("real GTK Paned");
            assert_eq!(
                split.orientation(),
                match axis {
                    SplitAxis::Horizontal => gtk::Orientation::Horizontal,
                    SplitAxis::Vertical => gtk::Orientation::Vertical,
                }
            );
            assert_orientation(&split.start_child().unwrap(), first);
            assert_orientation(&split.end_child().unwrap(), second);
        }
    }
}

fn leaves(widget: &gtk::Widget, output: &mut Vec<gtk::Widget>) {
    if widget.has_css_class("pane-surface") {
        output.push(widget.clone());
        return;
    }
    let mut child = widget.first_child();
    while let Some(w) = child {
        leaves(&w, output);
        child = w.next_sibling();
    }
}
