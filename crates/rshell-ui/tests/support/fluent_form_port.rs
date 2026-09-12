use rshell_core::{CatalogMutation, UiCommand, UiCommandPort, UiPortError};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug)]
pub(super) struct Receipt {
    pub(super) kind: &'static str,
    pub(super) id: Option<String>,
    pub(super) count: usize,
}

#[derive(Default)]
pub(super) struct RecordingPort {
    receipts: Mutex<Vec<Receipt>>,
    pub(super) reject: AtomicBool,
}
impl RecordingPort {
    pub(super) fn count(&self, kind: &str) -> usize {
        self.receipts
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.kind == kind)
            .count()
    }
    pub(super) fn last(&self) -> Receipt {
        self.receipts.lock().unwrap().last().unwrap().clone()
    }
}
impl UiCommandPort for RecordingPort {
    fn try_send(&self, command: UiCommand) -> Result<(), UiPortError> {
        let (kind, id, count) = match command {
            UiCommand::ApplyCatalog {
                mutation: CatalogMutation::Create(profile),
                ..
            } => ("create", Some(profile.id.0.to_string()), 1),
            UiCommand::ApplyCatalog {
                mutation: CatalogMutation::Update(profile),
                ..
            } => ("update", Some(profile.id.0.to_string()), 1),
            UiCommand::SaveTerminalProfile(profile) => {
                ("profile", Some(profile.id.0.to_string()), 1)
            }
            UiCommand::SaveSettings(_) => ("settings", None, 1),
            UiCommand::PreviewImport { .. } => ("preview", None, 1),
            UiCommand::CommitImport { preview, selected } => {
                ("commit", Some(preview.0.to_string()), selected.len())
            }
            UiCommand::CancelImport { preview } => ("cancel", Some(preview.0.to_string()), 1),
            UiCommand::Shutdown => ("shutdown", None, 1),
            _ => panic!("unexpected command kind in form fixture"),
        };
        self.receipts
            .lock()
            .unwrap()
            .push(Receipt { kind, id, count });
        println!(
            "FLUENT_COMMAND kind={kind} count={count} rejected={}",
            self.reject.load(Ordering::Relaxed)
        );
        if self.reject.load(Ordering::Relaxed) {
            Err(UiPortError::Busy)
        } else {
            Ok(())
        }
    }
}
