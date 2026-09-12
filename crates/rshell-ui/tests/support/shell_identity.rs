use super::fluent_native::{capture, close_window, descendants, field_for_label, wait_for_frame};
use super::{AcceptingPort, find_by_css_class, visual_fixture};
use gtk::prelude::*;
use relm4::{Component, ComponentController};
use rshell_ui::{MainWindow, MainWindowInit};
use std::sync::Arc;

pub(super) fn run() {
    let main = MainWindow::builder()
        .launch(MainWindowInit::new(
            Arc::new(AcceptingPort),
            visual_fixture(),
        ))
        .detach();
    main.widget().set_default_size(1360, 860);
    main.widget().present();
    wait_for_frame(main.widget(), "identity initial frame", |root| {
        find_by_css_class(root, "shell-standard").is_some()
    });
    let root = main.widget().upcast_ref::<gtk::Widget>();
    let sidebar = find_by_css_class(root, "sidebar").unwrap();
    let canvas = find_by_css_class(root, "terminal-canvas").unwrap();
    let pane = find_by_css_class(root, "pane-surface").unwrap();
    let active = find_by_css_class(root, "active-tab").unwrap();
    let search = find_by_css_class(root, "connection-search")
        .unwrap()
        .downcast::<gtk::SearchEntry>()
        .unwrap();
    let searched = std::rc::Rc::new(std::cell::Cell::new(false));
    let observed = searched.clone();
    search.connect_search_changed(move |_| observed.set(true));
    search.set_text("Visual");
    wait_for_frame(
        main.widget(),
        "sidebar search applied before selection",
        move |_| searched.get(),
    );
    let list = find_by_css_class(root, "connection-list")
        .unwrap()
        .downcast::<gtk::ListBox>()
        .unwrap();
    list.select_row(list.row_at_index(0).as_ref());
    wait_for_frame(main.widget(), "retained sidebar selection", |root| {
        find_by_css_class(root, "navigation-selected").is_some()
    });
    key(
        &canvas,
        gtk::gdk::Key::f,
        gtk::gdk::ModifierType::CONTROL_MASK | gtk::gdk::ModifierType::SHIFT_MASK,
    );
    wait_for_frame(main.widget(), "terminal search opened", |root| {
        find_by_css_class(root, "terminal-search").is_some_and(|w| w.is_mapped())
    });
    let terminal_search = find_by_css_class(root, "terminal-search")
        .unwrap()
        .downcast::<gtk::SearchEntry>()
        .unwrap();
    terminal_search.set_text("fixture");
    for (mode, w, h) in [
        ("compact", 800, 600),
        ("standard", 1360, 860),
        ("wide", 1920, 1080),
        ("compact", 800, 600),
    ] {
        resize(main.widget(), mode, w, h);
        assert_eq!(find_by_css_class(root, "sidebar").unwrap(), sidebar);
        assert_eq!(find_by_css_class(root, "terminal-canvas").unwrap(), canvas);
        assert_eq!(find_by_css_class(root, "pane-surface").unwrap(), pane);
        assert_eq!(find_by_css_class(root, "active-tab").unwrap(), active);
        assert_eq!(search.text(), "Visual");
        assert!(find_by_css_class(root, "navigation-selected").is_some());
        assert_eq!(terminal_search.text(), "fixture");
        assert!(terminal_search.is_mapped());
    }
    resize(main.widget(), "standard", 1360, 860);
    let create = descendants(&sidebar)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Button>().ok())
        .find(|b| b.tooltip_text().as_deref() == Some("Create a connection"))
        .unwrap();
    create.emit_clicked();
    wait_for_frame(main.widget(), "draft open", |root| {
        find_by_css_class(root, "editor-dialog").is_some_and(|w| w.is_mapped())
    });
    let name = field_for_label(root, "Name")
        .downcast::<gtk::Entry>()
        .unwrap();
    name.set_text("Unsubmitted fixture draft");
    for (mode, w, h) in [
        ("compact", 800, 600),
        ("standard", 1360, 860),
        ("wide", 1920, 1080),
    ] {
        resize(main.widget(), mode, w, h);
        assert_eq!(name.text(), "Unsubmitted fixture draft");
        assert_eq!(
            field_for_label(root, "Name"),
            name.clone().upcast::<gtk::Widget>()
        );
        assert_eq!(find_by_css_class(root, "terminal-canvas").unwrap(), canvas);
        capture(main.widget(), mode, "shell-retained-draft");
    }
    let modal = find_by_css_class(root, "editor-dialog").unwrap();
    key(
        &modal,
        gtk::gdk::Key::Escape,
        gtk::gdk::ModifierType::empty(),
    );
    wait_for_frame(main.widget(), "draft cancel", |root| {
        !find_by_css_class(root, "editor-dialog")
            .unwrap()
            .is_mapped()
    });
    assert_eq!(terminal_search.text(), "fixture");
    println!(
        "SHELL_IDENTITY_PASS real_modes=compact,standard,wide,compact tab=true pane=true terminal_session_widget=true sidebar_search=true terminal_search=true sidebar_selection=true draft=true"
    );
    close_window(main.widget());
}

fn resize(window: &gtk::ApplicationWindow, mode: &str, width: i32, height: i32) {
    window.set_default_size(width, height);
    let class = format!("shell-{mode}");
    wait_for_frame(window, "real breakpoint resize", move |root| {
        find_by_css_class(root, &class).is_some()
            && (root.width() - width).abs() <= 2
            && (root.height() - height).abs() <= 2
    });
}

fn key(widget: &gtk::Widget, key: gtk::gdk::Key, modifiers: gtk::gdk::ModifierType) {
    let controllers = widget.observe_controllers();
    let handled = (0..controllers.n_items())
        .filter_map(|i| controllers.item(i))
        .filter_map(|c| c.downcast::<gtk::EventControllerKey>().ok())
        .any(|c| c.emit_by_name::<bool>("key-pressed", &[&key, &0u32, &modifiers]));
    assert!(
        handled,
        "production keyboard controller must handle fixture key"
    );
}
