#![cfg(not(target_os = "macos"))]

use gtk::prelude::*;
use relm4::ComponentController;
use rshell_core::{
    AppEvent, AuthenticationKind, ImportCandidateId, ImportCandidateView, ImportPreviewId,
    ImportPreviewView, ImportSourceKind,
};
use rshell_ui::{ConnectionSidebarOutput, MainWindowMsg, apply_global_css};

#[path = "support/fluent_forms.rs"]
mod fluent_forms;
#[path = "support/fluent_interactions.rs"]
mod fluent_interactions;
#[path = "support/fluent_measure.rs"]
mod fluent_measure;
#[path = "support/fluent_native.rs"]
mod fluent_native;
use fluent_native::*;

#[test]
fn fluent_polish_renders_native_fields_titles_and_action_states() {
    gtk::init().expect("Fluent native proof requires an available GTK display; do not skip");
    apply_global_css();
    assert_css_parses();
    let settings = gtk::Settings::default().unwrap();
    settings.set_property("gtk-enable-animations", false);
    settings.set_property("gtk-cursor-blink", false);
    assert_wait_deadline();
    let mut failures = Vec::new();
    for (mode, width, height) in [
        ("compact", 800, 600),
        ("standard", 1_360, 860),
        ("wide", 1_920, 1_080),
    ] {
        fluent_interactions::run(mode, width, height);
        editor(mode, width, height, &mut failures);
        settings_form(mode, width, height, &mut failures);
        import(mode, width, height, &mut failures);
        fluent_forms::run(mode, width, height);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    println!("FLUENT_NATIVE_PASS cases=3 note_states=2 import_states=4");
}

fn assert_css_parses() {
    let errors = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let seen = errors.clone();
    let provider = gtk::CssProvider::new();
    provider.connect_parsing_error(move |_, _, error| seen.borrow_mut().push(error.to_string()));
    provider.load_from_data(rshell_ui::embedded_theme_css());
    assert!(
        errors.borrow().is_empty(),
        "production CSS parse errors: {:?}",
        errors.borrow()
    );
    println!("FLUENT_CSS_PARSE_PASS errors=0 embedded=true");
}

fn editor(mode: &str, width: i32, height: i32, failures: &mut Vec<String>) {
    let main = launch(width, height);
    main.emit(MainWindowMsg::Sidebar(ConnectionSidebarOutput::OpenCreate(
        None,
    )));
    wait_for_frame(main.widget(), "editor open", |root| {
        modal_ready(root, "editor-dialog")
    });
    let root = main.widget().upcast_ref();
    fluent_measure::record(root, mode, "editor-dialog");
    let title = label(root, "New connection");
    let description = title.layout().context().font_description().unwrap();
    println!("FLUENT_TITLE requested_case={mode} font={description}");
    check(
        description.size() == 18 * gtk::pango::SCALE && description.is_size_absolute(),
        format!("{mode}: connection title must be 18 logical px, got {description}"),
        failures,
    );
    let note = descendants(root)
        .into_iter()
        .find(|w| w.is::<gtk::TextView>())
        .unwrap();
    let default = pixels(&note);
    check(
        is_control(default.at(4, default.height / 2)),
        format!("{mode}: default Note surround is not surface-control"),
        failures,
    );
    capture(main.widget(), mode, "editor-default");
    assert!(note.grab_focus());
    wait_for_frame(&note, "Note focus", focus_within);
    let focused = pixels(&note);
    check(
        is_control(focused.at(4, focused.height / 2)),
        format!("{mode}: focused Note surround is not surface-control"),
        failures,
    );
    check(
        is_accent(focused.at(1, focused.height / 2)),
        format!("{mode}: Note lacks inset accent focus"),
        failures,
    );
    capture(main.widget(), mode, "editor-focus");
    close_window(main.widget());
}

fn settings_form(mode: &str, width: i32, height: i32, failures: &mut Vec<String>) {
    let main = launch(width, height);
    main.emit(MainWindowMsg::OpenSettings);
    wait_for_frame(main.widget(), "settings open", |root| {
        modal_ready(root, "settings-window")
    });
    let root = main.widget().upcast_ref();
    fluent_measure::record(root, mode, "settings-window");
    let first = field_for_label(root, "Default profile");
    let second = field_for_label(root, "Terminal profile");
    for text in ["Default profile", "Terminal profile"] {
        assert_eq!(
            label(root, text).xalign(),
            1.0,
            "shared labels stay right-aligned"
        );
    }
    let first_bounds = first.compute_bounds(root).unwrap();
    let second_bounds = second.compute_bounds(root).unwrap();
    println!(
        "FLUENT_COLUMNS requested_case={mode} delta={}",
        (first_bounds.x() - second_bounds.x()).abs()
    );
    check(
        (first_bounds.x() - second_bounds.x()).abs() <= 1.0,
        format!(
            "{mode}: Settings input columns drift: {} vs {}",
            first_bounds.x(),
            second_bounds.x()
        ),
        failures,
    );
    capture(main.widget(), mode, "settings");
    close_window(main.widget());
}

fn import(mode: &str, width: i32, height: i32, failures: &mut Vec<String>) {
    let main = launch(width, height);
    main.emit(MainWindowMsg::OpenImport);
    wait_for_frame(main.widget(), "import open", |root| {
        modal_ready(root, "import-dialog")
    });
    let root = main.widget().upcast_ref();
    fluent_measure::record(root, mode, "import-dialog");
    let commit = button(root, "Import selected");
    assert!(!commit.is_sensitive());
    let empty = pixels(commit.upcast_ref());
    check(
        !is_accent(empty.at(8, empty.height / 2)),
        format!("{mode}: disabled Import action is accent-filled"),
        failures,
    );
    let instruction = label(root, "Choose an import source to build a preview");
    let contrast = pixels(instruction.upcast_ref()).maximum_text_contrast();
    println!("FLUENT_INSTRUCTION requested_case={mode} contrast={contrast:.2}:1");
    check(
        contrast >= 4.5,
        format!("{mode}: rendered import instruction contrast {contrast:.2}:1 < 4.5:1"),
        failures,
    );
    capture(main.widget(), mode, "import-empty");
    main.emit(MainWindowMsg::AppEvent(AppEvent::ImportPreview(preview())));
    wait_for_frame(&commit, "import selected", |button| button.is_sensitive());
    assert!(commit.is_sensitive());
    let selected = pixels(commit.upcast_ref());
    check(
        is_accent(selected.at(8, selected.height / 2)),
        format!("{mode}: enabled Import must retain accent"),
        failures,
    );
    capture(main.widget(), mode, "import-selected");
    let toggle = descendants(root)
        .into_iter()
        .filter(|w| w.is_mapped())
        .find_map(|w| w.downcast::<gtk::CheckButton>().ok())
        .unwrap();
    toggle.set_active(false);
    wait_for_frame(&commit, "import deselected", |button| {
        !button.is_sensitive()
    });
    assert!(!commit.is_sensitive());
    let unselected = pixels(commit.upcast_ref());
    check(
        !is_accent(unselected.at(8, unselected.height / 2)),
        format!("{mode}: deselected Import remains accent-filled"),
        failures,
    );
    toggle.set_active(true);
    wait_for_frame(&commit, "import reselected", |button| button.is_sensitive());
    commit.emit_clicked();
    wait_for_frame(&commit, "import pending", |button| !button.is_sensitive());
    assert!(!commit.is_sensitive());
    let pending = pixels(commit.upcast_ref());
    check(
        !is_accent(pending.at(8, pending.height / 2)),
        format!("{mode}: pending Import remains accent-filled"),
        failures,
    );
    capture(main.widget(), mode, "import-pending");
    close_window(main.widget());
}

fn preview() -> ImportPreviewView {
    ImportPreviewView {
        id: ImportPreviewId::new(),
        source: ImportSourceKind::OpenSshConfig,
        groups: Vec::new(),
        warnings: Vec::new(),
        candidates: vec![ImportCandidateView {
            id: ImportCandidateId::new(),
            name: "Visual fixture".into(),
            host: "host.example.test".into(),
            port: 22,
            username: "operator".into(),
            source_label: "Visual fixture".into(),
            has_secret: false,
            selectable: true,
            authentication: AuthenticationKind::Agent,
            credential_reference_present: false,
            terminal_override_present: false,
            importable: true,
            wildcard: false,
            warnings: Vec::new(),
        }],
    }
}

fn check(condition: bool, message: String, failures: &mut Vec<String>) {
    if !condition {
        failures.push(message);
    }
}
