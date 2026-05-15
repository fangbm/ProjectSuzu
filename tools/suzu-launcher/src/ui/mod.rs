mod krkr_tab;
mod legal_notice;
mod preview_panel;
mod project_tab;
mod xp3_tab;

use eframe::egui;
use suzu_asset::{AssetType, KrkrCompatibilityReport};

use crate::app::LauncherApp;
use crate::preview::{fit_size, render_frame};

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.load_dropped_path(ctx);
        egui::TopBottomPanel::top("top").show(ctx, |ui| self.header(ui));
        egui::SidePanel::left("project")
            .resizable(true)
            .default_width(360.0)
            .show(ctx, |ui| self.project_panel(ui));
        egui::SidePanel::right("xp3")
            .resizable(true)
            .default_width(420.0)
            .show(ctx, |ui| self.xp3_panel(ui));
        egui::CentralPanel::default().show(ctx, |ui| self.game_panel(ui));
    }
}
