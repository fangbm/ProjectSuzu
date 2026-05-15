use eframe::egui;
use suzu_asset::{Xp3Options, Xp3PluginModule};

use crate::app::Xp3ViewerApp;
use crate::cli::XP3_PLUGIN_AUTHORIZATION_MESSAGE;
use crate::paths::clean_path_input;

impl Xp3ViewerApp {
    pub(crate) fn xp3_options(&self) -> Result<Xp3Options, String> {
        let module_path = clean_path_input(&self.xp3_plugin_path);
        if !module_path.is_empty() {
            self.ensure_xp3_plugin_authorized()?;
            let module = Xp3PluginModule::from_json_file(&module_path)
                .map_err(|error| format!("Failed to load XP3 plugin module: {error:#}"))?;
            return Ok(module.xp3_options());
        }
        Ok(Xp3Options::default())
    }

    pub(crate) fn xp3_plugin_requires_authorization(&self) -> bool {
        !clean_path_input(&self.xp3_plugin_path).is_empty()
    }

    pub(crate) fn ensure_xp3_plugin_authorized(&self) -> Result<(), String> {
        if self.xp3_plugin_requires_authorization() && !self.xp3_plugin_authorized {
            return Err(XP3_PLUGIN_AUTHORIZATION_MESSAGE.to_owned());
        }
        Ok(())
    }

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
