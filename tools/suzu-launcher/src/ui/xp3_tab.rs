use super::*;

impl LauncherApp {
    pub(crate) fn xp3_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("XP3 Import");
        ui.horizontal(|ui| {
            ui.label("XP3");
            let response = ui.text_edit_singleline(&mut self.xp3_path);
            if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                self.load_xp3();
            }
            if ui.button("Import").clicked() {
                self.load_xp3();
            }
        });
        ui.horizontal(|ui| {
            ui.label("XP3 plugin");
            ui.text_edit_singleline(&mut self.xp3_plugin_path);
            if ui.button("Run Selected Script").clicked() {
                self.start_xp3_script();
            }
        });
        self.xp3_plugin_authorization_ui(ui);

        ui.separator();
        ui.label(format!("{} entries", self.xp3_entries.len()));
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (index, entry) in self.xp3_entries.iter().enumerate() {
                if !matches!(entry.kind, AssetType::Script | AssetType::Data) {
                    continue;
                }
                let lock = if entry.protected { "locked" } else { "plain" };
                let label = format!(
                    "{:?} · {lock} · {} bytes · {}",
                    entry.kind, entry.original_size, entry.name
                );
                if ui
                    .selectable_label(self.selected_xp3_script == Some(index), label)
                    .clicked()
                {
                    self.selected_xp3_script = Some(index);
                }
            }
        });

        self.krkr_panel(ui);
    }
}
