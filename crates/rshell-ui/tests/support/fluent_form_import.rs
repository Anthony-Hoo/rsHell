use super::{actions::*, fluent_native::*, layout, port::RecordingPort};
use gtk::prelude::*;
use relm4::ComponentController;
use rshell_core::{AppEvent, AppFailure, AppFailureCategory, ImportReportView, ImportWarningView};
use rshell_ui::MainWindowMsg;
use std::sync::{Arc, atomic::Ordering};

pub(super) fn run(mode: &str, width: i32, height: i32) {
    let port = Arc::new(RecordingPort::default());
    let main = launch_with_port(width, height, port.clone());
    let root = main.widget().upcast_ref();
    let (modal, trigger) = open(root, "Import connections", "import-dialog");
    let commit = button(root, "Import selected");
    assert!(!commit.is_sensitive());
    state(main.widget(), &modal, mode, "forms-import-empty");
    button(root, "OpenSSH config").emit_clicked();
    let seen = port.clone();
    wait_for_frame(main.widget(), "production file callback", move |_| {
        seen.count("preview") == 1
    });
    let mut preview = super::super::preview();
    preview.warnings.push(ImportWarningView {
        code: "fixture-warning".into(),
        message: "Review this synthetic import warning before continuing.".into(),
    });
    let id = preview.id;
    main.emit(MainWindowMsg::AppEvent(AppEvent::ImportPreview(
        preview.clone(),
    )));
    wait_for_frame(&commit, "selected preview accepted", |w| w.is_sensitive());
    let endpoint = label(root, "operator@host.example.test:22");
    let contrast = pixels(endpoint.upcast_ref()).maximum_text_contrast();
    println!("FLUENT_ENDPOINT_CONTRAST mode={mode} ratio={contrast:.2}");
    assert!(
        contrast >= 4.5,
        "rendered import metadata contrast {contrast:.2}:1 < 4.5:1"
    );
    state(main.widget(), &modal, mode, "forms-import-selected-warning");
    toggle(&modal).set_active(false);
    wait_for_frame(&commit, "deselected", |w| !w.is_sensitive());
    assert_eq!(port.count("commit"), 0);
    state(main.widget(), &modal, mode, "forms-import-deselected");
    toggle(&modal).set_active(true);
    wait_for_frame(&commit, "reselected", |w| w.is_sensitive());
    port.reject.store(true, Ordering::Relaxed);
    commit.emit_clicked();
    let seen = port.clone();
    wait_for_frame(&commit, "Busy rejection", move |w| {
        w.is_sensitive() && seen.count("commit") == 1
    });
    wait_error(root);
    state(main.widget(), &modal, mode, "forms-import-rejected");
    port.reject.store(false, Ordering::Relaxed);
    commit.emit_clicked();
    let seen = port.clone();
    wait_for_frame(&commit, "commit pending", move |w| {
        !w.is_sensitive() && seen.count("commit") == 2
    });
    assert_eq!(port.last().id, Some(id.0.to_string()));
    assert_eq!(port.last().count, 1);
    assert!(!toggle(&modal).is_sensitive());
    assert!(!button(root, "OpenSSH config").is_sensitive());
    state(main.widget(), &modal, mode, "forms-import-pending");
    main.emit(MainWindowMsg::AppEvent(AppEvent::OperationFailed(
        AppFailure::fatal(AppFailureCategory::Validation, layout::LONG_ERROR),
    )));
    wait_error(root);
    layout::bottom(&modal);
    layout::long_error(&modal, &error(root));
    state(main.widget(), &modal, mode, "forms-import-long-error");
    button(root, "Preview again").emit_clicked();
    let seen = port.clone();
    wait_for_frame(&commit, "retry dispatch clears preview", move |w| {
        !w.is_sensitive() && seen.count("preview") == 2
    });
    state(main.widget(), &modal, mode, "forms-import-retry");
    main.emit(MainWindowMsg::AppEvent(AppEvent::ImportPreview(
        preview.clone(),
    )));
    wait_for_frame(&commit, "retry preview accepted", |w| w.is_sensitive());
    commit.emit_clicked();
    let seen = port.clone();
    wait_for_frame(&commit, "retry commit pending", move |w| {
        !w.is_sensitive() && seen.count("commit") == 3
    });
    main.emit(MainWindowMsg::AppEvent(AppEvent::ImportCompleted(
        ImportReportView {
            imported_connections: 1,
            ..Default::default()
        },
    )));
    wait_for_frame(main.widget(), "import completion", |w| {
        descendants(w)
            .iter()
            .filter_map(|w| w.clone().downcast::<gtk::Label>().ok())
            .any(|l| l.has_css_class("import-result") && l.text().contains("1 connections"))
    });
    assert!(!commit.is_sensitive());
    layout::bottom(&modal);
    let result = descendants(&modal)
        .into_iter()
        .find(|w| w.has_css_class("import-result"))
        .unwrap();
    layout::visible_in_body(&modal, &result);
    state(main.widget(), &modal, mode, "forms-import-success");
    escape(root, &modal, &trigger);
    assert_eq!(
        port.count("cancel"),
        0,
        "completed preview has no cancel command"
    );

    let (modal, trigger) = open(root, "Import connections", "import-dialog");
    button(root, "Legacy rsHell JSON").emit_clicked();
    let seen = port.clone();
    wait_for_frame(main.widget(), "second source dispatch", move |_| {
        seen.count("preview") == 3
    });
    preview.source = rshell_core::ImportSourceKind::LegacyRshellJson;
    main.emit(MainWindowMsg::AppEvent(AppEvent::ImportPreview(preview)));
    wait_for_frame(&commit, "cancel preview accepted", |w| w.is_sensitive());
    button(root, "Cancel").emit_clicked();
    closed(root, &modal, &trigger);
    assert_eq!(port.count("cancel"), 1);
    assert_eq!(port.last().id, Some(id.0.to_string()));
    main.emit(MainWindowMsg::AppEvent(AppEvent::ImportCancelled(id)));
    wait_for_frame(main.widget(), "cancel acknowledged", |w| {
        descendants(w)
            .iter()
            .filter_map(|w| w.clone().downcast::<gtk::Label>().ok())
            .any(|l| l.text() == "Import cancelled")
    });
    assert_eq!(
        port.count("cancel"),
        1,
        "cancel acknowledgement must not resend"
    );
    capture(main.widget(), mode, "forms-import-cancelled");
    close_window(main.widget());
}

fn toggle(modal: &gtk::Widget) -> gtk::CheckButton {
    descendants(modal)
        .into_iter()
        .find_map(|w| w.downcast().ok())
        .unwrap()
}
