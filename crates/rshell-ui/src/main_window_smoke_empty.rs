use gtk::prelude::*;
use relm4::gtk;

use crate::{MainWindow, SessionTabBarMsg, SmokeVisualState};

impl MainWindow {
    pub(crate) fn begin_empty_smoke_checkpoint(&self) {
        let tabs = self
            .view_model
            .workspace
            .tabs
            .iter()
            .map(|tab| tab.id)
            .collect::<Vec<_>>();
        for tab in tabs {
            self.send_tab(SessionTabBarMsg::Close(tab));
        }
    }

    pub(crate) fn empty_smoke_checkpoint_ready(&self) -> bool {
        self.view_model.workspace.tabs.is_empty()
            && !crate::visual_accessibility::descendants_including(self.shell.overlay.upcast_ref())
                .iter()
                .any(|widget| widget.is_mapped() && widget.has_css_class("terminal-canvas"))
    }

    pub(crate) fn smoke_modal_surface_open(&self) -> bool {
        self.modal.open_kind().is_some()
            || crate::visual_accessibility::descendants_including(self.shell.overlay.upcast_ref())
                .iter()
                .any(|widget| widget.is_mapped() && widget.has_css_class("content-dialog"))
    }

    pub(crate) fn prepare_empty_smoke_focus(&self) -> bool {
        let widgets =
            crate::visual_accessibility::descendants_including(self.shell.overlay.upcast_ref());
        let Some(button) = widgets
            .iter()
            .filter_map(|widget| widget.downcast_ref::<gtk::Button>())
            .find(|button| {
                button.is_mapped()
                    && button.width() > 0
                    && button.height() > 0
                    && button.is_sensitive()
                    && button.is_focusable()
                    && has_ancestor_class(button.upcast_ref(), "command-bar")
            })
        else {
            return false;
        };
        let focused = self
            .shell
            .overlay
            .root()
            .and_then(|root| gtk::prelude::RootExt::focus(&root));
        if button.has_focus() && focused.as_ref() == Some(button.upcast_ref()) {
            return true;
        }
        if button.grab_focus() {
            button.queue_draw();
            self.shell.overlay.queue_draw();
        }
        false
    }
}

pub(crate) fn modal_visual_state(state: SmokeVisualState) -> bool {
    matches!(
        state,
        SmokeVisualState::Editor
            | SmokeVisualState::Settings
            | SmokeVisualState::Import
            | SmokeVisualState::HostKey
            | SmokeVisualState::Authentication
    )
}

fn has_ancestor_class(widget: &gtk::Widget, class: &str) -> bool {
    let mut current = widget.parent();
    while let Some(widget) = current {
        if widget.has_css_class(class) {
            return true;
        }
        current = widget.parent();
    }
    false
}
