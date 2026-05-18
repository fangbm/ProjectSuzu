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

        ui.horizontal(|ui| {
            if ui.button("Run").clicked() {
                self.start_project_entry();
            }
            if ui.button("Check").clicked() {
                self.check_project();
            }
            if ui.button("Open Editor").clicked() {
                self.open_project_in_editor();
            }
        });

        ui.separator();
        if let Some(project) = self.project.clone() {
            if let Some(report) = &self.project_check {
                ui.label(format!("Entry: {}", report.entry_path.display()));
                ui.label(format!(
                    "{} assets · {} packages",
                    report.registered_assets, report.registered_packages
                ));
                ui.label(match &report.config_path {
                    Some(path) => format!("Config: {}", path.display()),
                    None => "Config: convention defaults".to_owned(),
                });
                for warning in report.warnings() {
                    ui.colored_label(egui::Color32::YELLOW, format!("Warning: {warning}"));
                }
            } else if project.root.join("scenario/main.szs").exists()
                || project.root.join("script/main.szs").exists()
            {
                ui.label("This folder can run by convention without game.suzu.toml.");
                if ui.button("Create game.suzu.toml").clicked() {
                    self.create_project_config();
                }
            } else {
                ui.label(
                    "Add scenario/main.szs or game.suzu.toml to make this a runnable project.",
                );
            }

            ui.separator();
            ui.label(format!(
                "{} scripts · {} resources",
                project.scripts.len(),
                project.resources.len()
            ));
            if ui.button("Run Selected Script").clicked() {
                self.start_project_script();
            }
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
