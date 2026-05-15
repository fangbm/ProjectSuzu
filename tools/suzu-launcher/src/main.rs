#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod cli;
mod conversion;
mod paths;
mod preview;
mod ui;

use eframe::egui;

use crate::app::LauncherApp;
use crate::cli::CliAction;

fn main() -> eframe::Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let initial = match cli::dispatch(&args) {
        Ok(CliAction::Handled) => return Ok(()),
        Ok(CliAction::LaunchGui { initial }) => initial,
        Err(error) => {
            if args
                .first()
                .and_then(|arg| arg.to_str())
                .is_some_and(|arg| arg == "--check")
            {
                eprintln!("Project Suzu Launcher check FAILED");
                eprintln!("reason: {error:#}");
            } else {
                eprintln!("{error:#}");
            }
            std::process::exit(1);
        }
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1220.0, 760.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Project Suzu Launcher",
        options,
        Box::new(move |cc| Box::new(LauncherApp::new(cc, initial))),
    )
}
