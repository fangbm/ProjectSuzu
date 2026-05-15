use std::time::Instant;

use super::*;
use suzu_platform::{DesktopApp, DesktopInputEvent};

impl LauncherApp {
    pub(crate) fn game_panel(&mut self, ui: &mut egui::Ui) {
        let mut stop = false;
        let Some(game) = &mut self.game else {
            ui.centered_and_justified(|ui| {
                ui.label("Open a project or XP3, then run a script.");
            });
            return;
        };

        ui.horizontal(|ui| {
            ui.heading(format!("Playing {}", game.label));
            if ui.button("Stop").clicked() {
                stop = true;
            }
        });
        if stop {
            self.game = None;
            return;
        }
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
        let desired = fit_size(egui::vec2(1280.0, 720.0), ui.available_size());
        let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());
        if response.clicked() {
            game.app.input(DesktopInputEvent::Confirm);
        }
        render_frame(ui.painter(), rect, &frame, &mut game.textures);
        ui.ctx().request_repaint();
    }
}
