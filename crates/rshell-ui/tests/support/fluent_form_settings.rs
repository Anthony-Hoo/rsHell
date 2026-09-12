use super::{actions::*, fluent_native::*, layout, port::RecordingPort};
use gtk::prelude::*;
use relm4::ComponentController;
use rshell_core::{
    AppEvent, AppFailure, AppFailureCategory, AppSettings, ColorScheme, TerminalProfile,
};
use rshell_ui::MainWindowMsg;
use std::sync::{Arc, atomic::Ordering};

pub(super) fn run(mode: &str, width: i32, height: i32) {
    let port = Arc::new(RecordingPort::default());
    let main = launch_with_port(width, height, port.clone());
    let root = main.widget().upcast_ref();
    let (modal, trigger) = open(root, "Terminal settings", "settings-window");
    let save_profile = button(root, "Save profile");
    let save_app = button(root, "Save defaults");
    assert!(!save_profile.is_sensitive() && !save_app.is_sensitive());
    state(main.widget(), &modal, mode, "forms-settings-disabled");
    layout::bottom(&modal);
    layout::visible_in_body(&modal, &entry(root, "Answerback"));
    state(main.widget(), &modal, mode, "forms-settings-last-field");
    layout::top(&modal);
    fill(&entry(root, "Profile name"), "Form fixture");
    let scheme = field_for_label(root, "Default color scheme")
        .downcast::<gtk::DropDown>()
        .unwrap();
    scheme.set_selected(1);
    wait_for_frame(&save_app, "defaults dirty", |w| w.is_sensitive());
    save_profile.emit_clicked();
    let seen = port.clone();
    wait_for_frame(&save_profile, "profile pending", move |w| {
        !w.is_sensitive() && seen.count("profile") == 1
    });
    assert!(!save_app.is_sensitive());
    assert_eq!(
        port.last().id,
        Some(TerminalProfile::default().id.0.to_string())
    );
    state(
        main.widget(),
        &modal,
        mode,
        "forms-settings-profile-pending",
    );
    main.emit(MainWindowMsg::AppEvent(AppEvent::TerminalProfilesChanged(
        vec![TerminalProfile {
            name: "Form fixture".into(),
            ..Default::default()
        }],
    )));
    wait_for_frame(&save_app, "profile accepted retains defaults draft", |w| {
        w.is_sensitive()
    });
    assert!(!save_profile.is_sensitive());
    save_app.emit_clicked();
    let seen = port.clone();
    wait_for_frame(&save_app, "defaults pending", move |w| {
        !w.is_sensitive() && seen.count("settings") == 1
    });
    state(
        main.widget(),
        &modal,
        mode,
        "forms-settings-defaults-pending",
    );
    main.emit(MainWindowMsg::AppEvent(AppEvent::SettingsChanged(
        AppSettings {
            color_scheme: ColorScheme::OneDark,
            ..Default::default()
        },
    )));
    wait_for_frame(main.widget(), "defaults accepted", |w| {
        descendants(w)
            .iter()
            .filter_map(|w| w.clone().downcast::<gtk::Label>().ok())
            .any(|l| l.text() == "Settings updated")
    });
    assert!(!save_app.is_sensitive() && !save_profile.is_sensitive());
    state(main.widget(), &modal, mode, "forms-settings-accepted");

    port.reject.store(true, Ordering::Relaxed);
    fill(&entry(root, "Profile name"), "Rejected fixture");
    save_profile.emit_clicked();
    wait_error(root);
    assert_eq!(port.count("profile"), 2);
    assert!(save_profile.is_sensitive());
    layout::bottom(&modal);
    state(
        main.widget(),
        &modal,
        mode,
        "forms-settings-profile-rejected",
    );
    scheme.set_selected(2);
    wait_for_frame(&save_app, "defaults changed for rejection", |w| {
        w.is_sensitive()
    });
    save_app.emit_clicked();
    let seen = port.clone();
    wait_for_frame(&save_app, "defaults rejected", move |w| {
        w.is_sensitive() && seen.count("settings") == 2
    });
    layout::bottom(&modal);
    state(
        main.widget(),
        &modal,
        mode,
        "forms-settings-defaults-rejected",
    );

    port.reject.store(false, Ordering::Relaxed);
    save_profile.emit_clicked();
    let seen = port.clone();
    wait_for_frame(&save_profile, "long validation pending", move |w| {
        !w.is_sensitive() && seen.count("profile") == 3
    });
    main.emit(MainWindowMsg::AppEvent(AppEvent::OperationFailed(
        AppFailure::fatal(AppFailureCategory::Validation, layout::LONG_ERROR),
    )));
    wait_error(root);
    layout::bottom(&modal);
    layout::long_error(&modal, &error(root));
    state(main.widget(), &modal, mode, "forms-settings-long-error");
    escape(root, &modal, &trigger);
    let (modal, trigger) = open(root, "Terminal settings", "settings-window");
    escape(root, &modal, &trigger);
    close_window(main.widget());
}
