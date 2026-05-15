use super::*;

use crate::cli::XP3_PLUGIN_AUTHORIZATION_MESSAGE;

impl LauncherApp {
    pub(crate) fn xp3_plugin_authorization_ui(&mut self, ui: &mut egui::Ui) {
        if self.xp3_plugin_requires_authorization() {
            ui.checkbox(
                &mut self.xp3_plugin_authorized,
                "I have rights to process these assets",
            );
            ui.label(XP3_PLUGIN_AUTHORIZATION_MESSAGE);
        } else {
            self.xp3_plugin_authorized = false;
        }
    }
}
