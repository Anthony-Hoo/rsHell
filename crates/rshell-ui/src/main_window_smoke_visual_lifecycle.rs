use gtk::prelude::*;

use crate::{
    MainWindow, SmokeVisualCheckpoint, SmokeVisualState,
    main_window_smoke_matrix::{focus_restored, press_escape},
    main_window_smoke_visual::VisualCheckpointPhase,
};

impl MainWindow {
    pub(crate) fn close_visual_checkpoint(
        &mut self,
        checkpoint: &SmokeVisualCheckpoint,
    ) -> Result<bool, &'static str> {
        if matches!(
            checkpoint.state,
            SmokeVisualState::Editor | SmokeVisualState::Settings | SmokeVisualState::Import
        ) {
            let surface = self
                .smoke_checkpoint_surface(checkpoint.state)
                .ok_or("visual_modal_unavailable")?;
            if !press_escape(&surface) {
                return Err("visual_escape_not_handled");
            }
            self.smoke_state.modal_escape_verified = true;
            self.smoke_state.visual_checkpoint = VisualCheckpointPhase::Closing;
            Ok(false)
        } else {
            self.smoke_state.visual_checkpoint = VisualCheckpointPhase::Complete;
            Ok(true)
        }
    }

    pub(crate) fn finish_visual_checkpoint(
        &mut self,
        checkpoint: &SmokeVisualCheckpoint,
    ) -> Result<bool, &'static str> {
        if self
            .smoke_checkpoint_surface(checkpoint.state)
            .is_some_and(|surface| surface.is_visible())
            || self.modal.open_kind().is_some()
            || !self.shell.background.is_sensitive()
        {
            return Ok(false);
        }
        let root = self.smoke_root()?;
        self.smoke_state.modal_focus_restore_verified = focus_restored(
            root.upcast_ref(),
            self.smoke_state.visual_focus_trigger.as_ref(),
        );
        if !self.smoke_state.modal_focus_restore_verified {
            return Ok(false);
        }
        let evidence = self
            .smoke_state
            .pending_visual
            .as_mut()
            .ok_or("visual_evidence_missing")?;
        if evidence.checkpoint_id != checkpoint.id {
            return Err("visual_evidence_mismatch");
        }
        evidence.accessibility.focus_restored = self.smoke_state.modal_focus_restore_verified;
        evidence.accessibility.escape_cancelled = self.smoke_state.modal_escape_verified;
        if !evidence.contract_passes() {
            return Err("visual_checkpoint_incomplete");
        }
        let evidence = self
            .smoke_state
            .pending_visual
            .take()
            .ok_or("visual_evidence_missing")?;
        self.smoke_state
            .visuals
            .insert(checkpoint.id.clone(), evidence);
        self.smoke_state.visual_checkpoint = VisualCheckpointPhase::Complete;
        self.smoke_state.visual_completion_tick_pending = true;
        Ok(true)
    }
}
