use super::{actions::*, fluent_native::*, layout, port::RecordingPort};
use gtk::prelude::*;
use relm4::ComponentController;
use rshell_core::{AppEvent, AppFailure, AppFailureCategory, ConnectionCatalog, ConnectionProfile};
use rshell_ui::{ConnectionSidebarOutput, MainWindowMsg};
use std::sync::{Arc, atomic::Ordering};

const NOTE: &str =
    "Fixture note\nSecond line\nThird line\nFourth line\nFifth line\nFinal note line";

pub(super) fn run(mode: &str, width: i32, height: i32) {
    let port = Arc::new(RecordingPort::default());
    let main = launch_with_port(width, height, port.clone());
    let root = main.widget().upcast_ref();
    let tooltip = if mode == "compact" {
        "New connection"
    } else {
        "Create a connection"
    };
    let (modal, trigger) = open(root, tooltip, "editor-dialog");
    let save = button(root, "Save connection");
    state(main.widget(), &modal, mode, "forms-editor-create");
    save.emit_clicked();
    wait_error(root);
    assert_eq!(
        port.count("create"),
        0,
        "invalid empty Host must not dispatch"
    );
    assert!(error(root).text().to_ascii_lowercase().contains("host"));
    layout::bottom(&modal);
    state(main.widget(), &modal, mode, "forms-editor-invalid-host");
    fill(&entry(root, "Name"), "Form fixture");
    fill(&entry(root, "Host"), "fixture.example.test");
    let note = descendants(&modal)
        .into_iter()
        .find_map(|w| w.downcast::<gtk::TextView>().ok())
        .unwrap();
    note.buffer().set_text(NOTE);
    layout::reveal(&modal, &note);
    assert!(note.grab_focus());
    wait_for_frame(&note, "populated Note focus", focus_within);
    state(main.widget(), &modal, mode, "forms-editor-note");
    let transport = field_for_label(root, "Transport")
        .downcast::<gtk::DropDown>()
        .unwrap();
    transport.set_selected(1);
    wait_for_frame(&transport, "native auth options", |_| true);
    let identity = entry(root, "Identity file");
    let secret = field_for_label(root, "Secret")
        .downcast::<gtk::PasswordEntry>()
        .unwrap();
    for (name, suffix, identity_on, secret_on) in [
        ("Password", "password", false, true),
        ("Public key", "key", true, true),
        ("Agent", "agent", false, false),
        ("Keyboard interactive", "keyboard", false, false),
    ] {
        let toggle = descendants(&modal)
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::CheckButton>().ok())
            .find(|w| w.label().as_deref() == Some(name))
            .unwrap();
        // Preserve the real transport capability matrix: Agent is System-only.
        transport.set_selected(if name == "Agent" { 0 } else { 1 });
        wait_for_frame(&toggle, "supported transport authentication", |w| {
            w.is_sensitive()
        });
        toggle.set_active(true);
        let id = identity.clone();
        let pw = secret.clone();
        wait_for_frame(&toggle, "authentication state", move |w| {
            w.clone()
                .downcast::<gtk::CheckButton>()
                .unwrap()
                .is_active()
                && id.is_sensitive() == identity_on
                && pw.is_sensitive() == secret_on
        });
        layout::reveal(&modal, &toggle.parent().unwrap());
        state(
            main.widget(),
            &modal,
            mode,
            &format!("forms-editor-auth-{suffix}"),
        );
    }
    let agent = descendants(&modal)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::CheckButton>().ok())
        .find(|w| w.label().as_deref() == Some("Agent"))
        .unwrap();
    transport.set_selected(0);
    wait_for_frame(&agent, "system Agent available", |w| w.is_sensitive());
    agent.set_active(true);
    let inherit = field_for_label(root, "Terminal type")
        .downcast::<gtk::CheckButton>()
        .unwrap();
    let grid = inherit.parent().unwrap().downcast::<gtk::Grid>().unwrap();
    let (_, row, _, _) = grid.query_child(&inherit);
    let value = grid
        .child_at(2, row)
        .unwrap()
        .downcast::<gtk::Entry>()
        .unwrap();
    assert!(inherit.is_active() && !value.is_sensitive());
    inherit.set_active(false);
    wait_for_frame(&value, "explicit override", |w| w.is_sensitive());
    fill(&value, "xterm-256color");
    layout::reveal(&modal, &inherit);
    state(main.widget(), &modal, mode, "forms-editor-override");
    button(root, "Inherit all terminal settings").emit_clicked();
    wait_for_frame(&value, "inherited override", |w| !w.is_sensitive());
    assert!(inherit.is_active());
    layout::bottom(&modal);
    layout::visible_in_body(&modal, &button(root, "Inherit all terminal settings"));
    state(
        main.widget(),
        &modal,
        mode,
        "forms-editor-inherit-last-field",
    );

    port.reject.store(true, Ordering::Relaxed);
    save.emit_clicked();
    let seen = port.clone();
    wait_for_frame(&save, "create Busy rejection", move |w| {
        w.is_sensitive() && seen.count("create") == 1
    });
    layout::bottom(&modal);
    state(main.widget(), &modal, mode, "forms-editor-rejected");
    port.reject.store(false, Ordering::Relaxed);
    save.emit_clicked();
    let seen = port.clone();
    wait_for_frame(&save, "create pending", move |w| {
        !w.is_sensitive() && seen.count("create") == 2
    });
    state(main.widget(), &modal, mode, "forms-editor-pending");
    main.emit(MainWindowMsg::AppEvent(AppEvent::OperationFailed(
        AppFailure::fatal(AppFailureCategory::Validation, layout::LONG_ERROR),
    )));
    wait_error(root);
    layout::bottom(&modal);
    // Editor adds a stable error-category prefix; its content remains wholly scrollable.
    let e = error(root);
    assert!(e.text().contains(layout::LONG_ERROR));
    assert!(e.ancestor(gtk::ScrolledWindow::static_type()).is_some());
    state(main.widget(), &modal, mode, "forms-editor-long-error");
    save.emit_clicked();
    let seen = port.clone();
    wait_for_frame(&save, "create retry pending", move |w| {
        !w.is_sensitive() && seen.count("create") == 3
    });
    let receipt = port.last();
    let mut profile = ConnectionProfile::new("Form fixture", "fixture.example.test");
    profile.id = rshell_core::ConnectionId(receipt.id.unwrap().parse().unwrap());
    profile.note = NOTE.into();
    let mut catalog = ConnectionCatalog::default();
    catalog.connections.insert(profile.id, profile.clone());
    main.emit(MainWindowMsg::AppEvent(AppEvent::CatalogChanged(
        catalog.clone(),
    )));
    closed(root, &modal, &trigger);

    // OpenEdit is the existing sidebar output boundary; the real production editor
    // and its field/save bindings are used, with the current native trigger focused.
    assert!(trigger.grab_focus());
    main.emit(MainWindowMsg::Sidebar(ConnectionSidebarOutput::OpenEdit(
        profile,
    )));
    wait_for_frame(root, "edit per-open initial focus", |w| {
        modal_ready(w, "editor-dialog")
    });
    let modal = layout::modal(root, "editor-dialog");
    assert_eq!(entry(root, "Name").text(), "Form fixture");
    assert_eq!(
        note.buffer().text(
            &note.buffer().start_iter(),
            &note.buffer().end_iter(),
            false
        ),
        NOTE
    );
    state(main.widget(), &modal, mode, "forms-editor-edit");
    fill(&entry(root, "Name"), "Edited fixture");
    button(root, "Save connection").emit_clicked();
    let seen = port.clone();
    wait_for_frame(&save, "update pending", move |w| {
        !w.is_sensitive() && seen.count("update") == 1
    });
    assert_eq!(
        port.last().id,
        Some(catalog.connections.keys().next().unwrap().0.to_string())
    );
    catalog.connections.values_mut().next().unwrap().name = "Edited fixture".into();
    main.emit(MainWindowMsg::AppEvent(AppEvent::CatalogChanged(catalog)));
    closed(root, &modal, &trigger);
    let (modal, trigger) = open(root, tooltip, "editor-dialog");
    escape(root, &modal, &trigger);
    close_window(main.widget());
}
