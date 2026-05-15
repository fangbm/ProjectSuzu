use super::*;

impl LauncherApp {
    pub(crate) fn header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Project Suzu Launcher");
            ui.separator();
            ui.label(&self.status);
        });
    }

    pub(crate) fn project_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Suzu Project");
        ui.horizontal(|ui| {
            ui.label("Folder");
            let response = ui.text_edit_singleline(&mut self.project_path);
            if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                self.scan_project();
            }
            if ui.button("Open").clicked() {
                self.scan_project();
            }
        });

        if ui.button("Run Selected Script").clicked() {
            self.start_project_script();
        }

        ui.separator();
        if let Some(project) = &self.project {
            ui.label(format!(
                "{} scripts · {} resources",
                project.scripts.len(),
                project.resources.len()
            ));
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, script) in project.scripts.iter().enumerate() {
                    if ui
                        .selectable_label(
                            self.selected_project_script == Some(index),
                            script.display().to_string(),
                        )
                        .clicked()
                    {
                        self.selected_project_script = Some(index);
                    }
                }
            });
        } else {
            ui.label("Drop a project folder here or paste its path.");
        }
    }
}
