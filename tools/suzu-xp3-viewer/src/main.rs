#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod cli;
mod fonts;
mod paths;
mod plugin;
mod preview;
mod ui;

use eframe::egui;

use crate::app::Xp3ViewerApp;
use crate::cli::CliAction;

fn main() -> eframe::Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let initial_path = match cli::dispatch(&args) {
        Ok(CliAction::Handled) => return Ok(()),
        Ok(CliAction::LaunchGui { initial_path }) => initial_path,
        Err(error) => {
            if args
                .first()
                .and_then(|arg| arg.to_str())
                .is_some_and(|arg| arg == "--check")
            {
                eprintln!("Project Suzu XP3 Viewer check FAILED");
                eprintln!("reason: {error:#}");
            } else {
                eprintln!("{error:#}");
            }
            std::process::exit(1);
        }
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1180.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Project Suzu XP3 Viewer",
        options,
        Box::new(move |cc| Box::new(Xp3ViewerApp::new(cc, initial_path))),
    )
}
