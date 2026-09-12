//! Extra production-form actions, invoked serially by the existing GTK libtest.
use super::fluent_native;
#[path = "fluent_form_actions.rs"]
pub(crate) mod actions;
#[path = "fluent_form_editor.rs"]
mod editor;
#[path = "fluent_form_import.rs"]
mod import;
#[path = "fluent_form_layout.rs"]
pub(crate) mod layout;
#[path = "fluent_form_port.rs"]
mod port;
#[path = "fluent_form_settings.rs"]
mod settings;

pub(crate) fn run(mode: &str, width: i32, height: i32) {
    editor::run(mode, width, height);
    settings::run(mode, width, height);
    import::run(mode, width, height);
    println!(
        "FLUENT_FORMS_PASS requested_mode={mode} production_bindings=true synthetic_port=true"
    );
}
