use std::time::Instant;

use eframe::egui;
use suzu_platform::{DesktopApp, DesktopInputEvent};

use crate::app::{Preview, Xp3ViewerApp};
use crate::preview::{fit_size, render_frame};

impl Xp3ViewerApp {
    pub(crate) fn top_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.heading("Project Suzu XP3 Viewer");
            ui.separator();
            ui.label("XP3");
            let response = ui.add_sized(
                [520.0, 22.0],
                egui::TextEdit::singleline(&mut self.xp3_path).hint_text(r"D:\game\data.xp3"),
            );
            if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                self.load_archive(ctx);
            }
            let busy = self.archive_job.is_some() || self.preview_job.is_some();
            let load_label = if self.archive_job.is_some() {
                "Loading..."
            } else if self.preview_job.is_some() {
                "Previewing..."
            } else {
                "Load"
            };
            if ui
                .add_enabled(!busy, egui::Button::new(load_label))
                .clicked()
            {
                self.load_archive(ctx);
            }
            let can_start = !busy && self.selected_script_id().is_some();
            if ui
                .add_enabled(can_start, egui::Button::new("Start Game"))
                .clicked()
            {
                self.start_game();
            }
            if self.game.is_some() && ui.button("Stop").clicked() {
                self.game = None;
                self.status = "Stopped game preview.".to_owned();
            }
        });
        ui.horizontal(|ui| {
            ui.label("XP3 plugin");
            ui.text_edit_singleline(&mut self.xp3_plugin_path);
            ui.separator();
            ui.label(&self.status);
        });
        self.xp3_plugin_authorization_ui(ui);
    }

    pub(crate) fn entries_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Entries");
        ui.label(format!("{} indexed", self.entries.len()));
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut clicked = None;
            let preview_busy = self.preview_job.is_some();
            for (index, row) in self.entries.iter().enumerate() {
                let selected = self.selected == Some(index);
                let marker = if row.protected { "locked" } else { "plain" };
                let label = format!(
                    "{:?} · {} · {} / {} bytes · {}",
                    row.kind, marker, row.packed_size, row.original_size, row.name
                );
                if ui
                    .add_enabled(!preview_busy, egui::SelectableLabel::new(selected, label))
                    .clicked()
                {
                    clicked = Some(index);
                }
            }
            if let Some(index) = clicked {
                self.select_entry(ctx, index);
            }
        });
    }

    pub(crate) fn preview_panel(&mut self, ui: &mut egui::Ui) {
        if self.game.is_some() {
            self.game_panel(ui);
            return;
        }

        ui.heading("Preview");
        ui.separator();

        match &self.preview {
            Preview::Empty => {
                ui.label("Load an XP3 and select an entry.");
            }
            Preview::Loading { name } => {
                ui.spinner();
                ui.label(format!("Loading preview for {name}..."));
            }
            Preview::Image {
                name,
                size,
                texture,
            } => {
                ui.label(format!("{} · {}x{}", name, size[0], size[1]));
                ui.add_space(8.0);
                let available = ui.available_size();
                let image_size = fit_size(
                    egui::vec2(size[0] as f32, size[1] as f32),
                    egui::vec2(available.x.max(1.0), available.y.max(1.0)),
                );
                ui.image((texture.id(), image_size));
            }
            Preview::Text {
                name,
                text,
                truncated,
            } => {
                ui.label(if *truncated {
                    format!("{name} · text preview truncated")
                } else {
                    format!("{name} · text")
                });
                ui.add(
                    egui::TextEdit::multiline(&mut text.as_str())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(28),
                );
            }
            Preview::Binary { name, bytes, kind } => {
                ui.label(format!("{name} · {:?} · {bytes} bytes", kind));
                ui.label("This entry loaded successfully but has no visual preview.");
            }
            Preview::Error { name, message } => {
                ui.colored_label(egui::Color32::from_rgb(210, 72, 72), name);
                ui.label(message);
            }
        }
    }

    pub(crate) fn game_panel(&mut self, ui: &mut egui::Ui) {
        let Some(game) = &mut self.game else {
            return;
        };

        ui.heading(format!("Playing `{}`", game.script_id));
        ui.separator();

        let now = Instant::now();
        let delta_ms = now
            .duration_since(game.last_frame)
            .as_millis()
            .clamp(0, u32::MAX as u128) as u32;
        game.last_frame = now;

        if ui.input(|input| {
            input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
        }) {
            game.app.input(DesktopInputEvent::Confirm);
        }
        if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            game.app.input(DesktopInputEvent::Cancel);
        }
        if ui.input(|input| input.key_pressed(egui::Key::ArrowDown)) {
            game.app
                .input(DesktopInputEvent::MoveSelection { delta: 1 });
        }
        if ui.input(|input| input.key_pressed(egui::Key::ArrowUp)) {
            game.app
                .input(DesktopInputEvent::MoveSelection { delta: -1 });
        }

        let frame = game.app.update(delta_ms.max(16));
        let available = ui.available_size();
        let desired = fit_size(egui::vec2(1280.0, 720.0), available);
        let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());
        if response.clicked() {
            game.app.input(DesktopInputEvent::Confirm);
        }

        render_frame(ui.painter(), rect, &frame, &mut game.textures);
        ui.ctx().request_repaint();
    }
}

impl eframe::App for Xp3ViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_background_jobs(ctx);
        self.load_dropped_xp3(ctx);
        egui::TopBottomPanel::top("top").show(ctx, |ui| self.top_bar(ui, ctx));
        egui::SidePanel::left("entries")
            .resizable(true)
            .default_width(430.0)
            .show(ctx, |ui| self.entries_panel(ui, ctx));
        egui::CentralPanel::default().show(ctx, |ui| self.preview_panel(ui));
    }
}
