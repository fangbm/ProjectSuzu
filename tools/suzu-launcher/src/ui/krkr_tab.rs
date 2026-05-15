use super::*;

impl LauncherApp {
    pub(crate) fn krkr_panel(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.heading("KRKR Package");
        ui.horizontal(|ui| {
            ui.label("Folder");
            let response = ui.text_edit_singleline(&mut self.krkr_path);
            if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                self.scan_krkr_package();
            }
            if ui.button("Scan").clicked() {
                self.scan_krkr_package();
            }
        });
        if let Some(report) = &self.krkr_report {
            ui.label(format!(
                "{} XP3 · {} entries · {} script-like · {} protected scripts",
                report.archives.len(),
                report.total_entries,
                report.total_scripts,
                report.protected_scripts
            ));
            ui.label(format!("Root: {}", report.root.display()));
            if let Some(compatibility) = &self.krkr_compatibility {
                if compatibility.has_protected_entries() {
                    ui.colored_label(
                        egui::Color32::from_rgb(210, 72, 72),
                        format!(
                            "Direct playback requires an external XP3 plugin for {} protected script-like entries.",
                            compatibility.protected_script_entries()
                        ),
                    );
                }
            }
            egui::ScrollArea::vertical()
                .max_height(190.0)
                .show(ui, |ui| {
                    for archive in &report.archives {
                        let name = archive
                            .path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("<xp3>");
                        if let Some(error) = &archive.error {
                            ui.colored_label(
                                egui::Color32::from_rgb(210, 72, 72),
                                format!("{name} · failed · {error}"),
                            );
                            continue;
                        }
                        ui.label(format!(
                            "{name} · {} MB · {} entries · {} scripts · {} protected",
                            archive.bytes / 1024 / 1024,
                            archive.entries,
                            archive.scripts,
                            archive.protected_scripts
                        ));
                        for candidate in &archive.candidates {
                            ui.label(format!("  entry: {candidate}"));
                        }
                    }
                });
            if report.protected_scripts > 0 {
                if self
                    .krkr_compatibility
                    .as_ref()
                    .is_some_and(KrkrCompatibilityReport::has_protected_entries)
                {
                    ui.colored_label(
                        egui::Color32::from_rgb(190, 92, 32),
                        "This package needs an external XP3 plugin before conversion.",
                    );
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(190, 92, 32),
                        "Protected KRKR script entries need an external XP3 plugin before conversion.",
                    );
                }
            }
        } else {
            ui.label("Paste a KRKR game folder or drop it into the launcher.");
        }
        ui.horizontal(|ui| {
            ui.label("Output");
            ui.text_edit_singleline(&mut self.krkr_output_path);
        });
        if ui.button("Convert KRKR Startup Flow").clicked() {
            self.convert_first_readable_krkr_script();
        }
    }
}
