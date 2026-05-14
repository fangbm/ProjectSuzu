#![cfg_attr(windows, windows_subsystem = "windows")]

mod cli;
mod conversion;
mod paths;
mod preview;

use std::{fs, path::PathBuf, time::Instant};

use eframe::egui;
use suzu_asset::{
    probe_krkr_directory, AssetType, KrkrCompatibilityReport, Xp3Archive, Xp3Entry, Xp3Options,
    Xp3PluginModule,
};
use suzu_editor_core::ProjectIndex;
use suzu_platform::{DesktopApp, DesktopInputEvent};

use crate::cli::{CliAction, XP3_PLUGIN_AUTHORIZATION_MESSAGE};
use crate::conversion::{convert_krkr_package_to_suzu_project, krkr_entry_looks_like_entrypoint};
use crate::paths::{
    asset_id_from_path, asset_type_from_path, clean_path_input, default_krkr_output_path,
    xp3_path_from_input,
};
use crate::preview::{fit_size, install_cjk_fonts, preview_app, render_frame, GamePreview};

fn main() -> eframe::Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let initial = match cli::dispatch(&args) {
        Ok(CliAction::Handled) => return Ok(()),
        Ok(CliAction::LaunchGui { initial }) => initial,
        Err(error) => {
            eprintln!("{error:#}");
            return Ok(());
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

struct LauncherApp {
    project_path: String,
    project: Option<ProjectIndex>,
    selected_project_script: Option<usize>,
    xp3_path: String,
    krkr_path: String,
    krkr_output_path: String,
    xp3_plugin_path: String,
    xp3_plugin_authorized: bool,
    krkr_report: Option<KrkrPackageReport>,
    krkr_compatibility: Option<KrkrCompatibilityReport>,
    xp3_entries: Vec<EntryRow>,
    selected_xp3_script: Option<usize>,
    game: Option<GamePreview>,
    status: String,
}

#[derive(Debug, Clone)]
struct EntryRow {
    name: String,
    kind: AssetType,
    protected: bool,
    original_size: u64,
}

#[derive(Debug, Clone)]
struct KrkrPackageReport {
    root: PathBuf,
    archives: Vec<KrkrArchiveSummary>,
    total_entries: usize,
    total_scripts: usize,
    protected_scripts: usize,
}

#[derive(Debug, Clone)]
struct KrkrArchiveSummary {
    path: PathBuf,
    bytes: u64,
    entries: usize,
    scripts: usize,
    protected_scripts: usize,
    candidates: Vec<String>,
    error: Option<String>,
}

impl LauncherApp {
    fn new(cc: &eframe::CreationContext<'_>, initial: PathBuf) -> Self {
        install_cjk_fonts(&cc.egui_ctx);
        let mut app = Self {
            project_path: String::new(),
            project: None,
            selected_project_script: None,
            xp3_path: String::new(),
            krkr_path: String::new(),
            krkr_output_path: String::new(),
            xp3_plugin_path: String::new(),
            xp3_plugin_authorized: false,
            krkr_report: None,
            krkr_compatibility: None,
            xp3_entries: Vec::new(),
            selected_xp3_script: None,
            game: None,
            status: "Open a Suzu project folder or import an XP3 archive.".to_owned(),
        };

        if !initial.as_os_str().is_empty() {
            if initial
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("xp3"))
            {
                app.xp3_path = initial.display().to_string();
                app.load_xp3();
            } else {
                app.project_path = initial.display().to_string();
                app.krkr_path = initial.display().to_string();
                app.krkr_output_path = default_krkr_output_path(&initial).display().to_string();
                app.scan_project();
            }
        }
        app
    }

    fn scan_project(&mut self) {
        let path = clean_path_input(&self.project_path);
        match ProjectIndex::scan(&path) {
            Ok(project) => {
                self.status = format!(
                    "Project loaded: {} scripts, {} resources.",
                    project.scripts.len(),
                    project.resources.len()
                );
                self.project_path = project.root.display().to_string();
                self.selected_project_script = (!project.scripts.is_empty()).then_some(0);
                self.project = Some(project);
                self.game = None;
            }
            Err(error) => {
                self.project = None;
                self.selected_project_script = None;
                self.status = format!("Failed to open project: {error:#}");
            }
        }
    }

    fn load_xp3(&mut self) {
        let path = match xp3_path_from_input(&self.xp3_path) {
            Ok(path) => path,
            Err(error) => {
                self.status = error;
                return;
            }
        };
        let options = match self.xp3_options() {
            Ok(options) => options,
            Err(error) => {
                self.status = error;
                return;
            }
        };

        match Xp3Archive::from_file_with_options(&path, options) {
            Ok(archive) => {
                self.xp3_path = path.display().to_string();
                self.xp3_entries = archive.entries().iter().map(EntryRow::from).collect();
                self.selected_xp3_script = self
                    .xp3_entries
                    .iter()
                    .position(|entry| entry.kind == AssetType::Script);
                self.status = format!("XP3 imported: {} entries.", self.xp3_entries.len());
                self.game = None;
            }
            Err(error) => {
                self.xp3_entries.clear();
                self.selected_xp3_script = None;
                self.status = format!("Failed to import XP3: {error:#}");
            }
        }
    }

    fn scan_krkr_package(&mut self) {
        let root = PathBuf::from(clean_path_input(&self.krkr_path));
        if self.krkr_output_path.trim().is_empty() {
            self.krkr_output_path = default_krkr_output_path(&root).display().to_string();
        }
        let options = match self.xp3_options() {
            Ok(options) => options,
            Err(error) => {
                self.krkr_report = None;
                self.krkr_compatibility = None;
                self.status = error;
                return;
            }
        };
        let mut archives = Vec::new();
        let mut total_entries = 0;
        let mut total_scripts = 0;
        let mut protected_scripts = 0;

        let read_dir = match fs::read_dir(&root) {
            Ok(read_dir) => read_dir,
            Err(error) => {
                self.krkr_report = None;
                self.krkr_compatibility = None;
                self.status = format!("Failed to scan KRKR folder: {error}");
                return;
            }
        };
        self.krkr_compatibility = probe_krkr_directory(&root).ok();

        for entry in read_dir.flatten() {
            let path = entry.path();
            if !path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("xp3"))
            {
                continue;
            }

            let bytes = entry.metadata().map(|meta| meta.len()).unwrap_or_default();
            let summary = match Xp3Archive::from_file_with_options(&path, options.clone()) {
                Ok(archive) => {
                    let rows = archive
                        .entries()
                        .iter()
                        .map(EntryRow::from)
                        .collect::<Vec<_>>();
                    let scripts = rows
                        .iter()
                        .filter(|row| matches!(row.kind, AssetType::Script | AssetType::Data))
                        .collect::<Vec<_>>();
                    let protected = scripts.iter().filter(|row| row.protected).count();
                    let candidates = scripts
                        .iter()
                        .filter(|row| krkr_entry_looks_like_entrypoint(&row.name))
                        .take(12)
                        .map(|row| row.name.clone())
                        .collect::<Vec<_>>();
                    total_entries += rows.len();
                    total_scripts += scripts.len();
                    protected_scripts += protected;
                    KrkrArchiveSummary {
                        path,
                        bytes,
                        entries: rows.len(),
                        scripts: scripts.len(),
                        protected_scripts: protected,
                        candidates,
                        error: None,
                    }
                }
                Err(error) => KrkrArchiveSummary {
                    path,
                    bytes,
                    entries: 0,
                    scripts: 0,
                    protected_scripts: 0,
                    candidates: Vec::new(),
                    error: Some(format!("{error:#}")),
                },
            };
            archives.push(summary);
        }

        archives.sort_by(|left, right| left.path.cmp(&right.path));
        self.status = format!(
            "KRKR scan complete: {} XP3 archives, {total_scripts} script-like entries.",
            archives.len()
        );
        self.krkr_report = Some(KrkrPackageReport {
            root,
            archives,
            total_entries,
            total_scripts,
            protected_scripts,
        });
    }

    fn convert_first_readable_krkr_script(&mut self) {
        if self.krkr_report.is_none() {
            self.scan_krkr_package();
        }
        let option_candidates = match self.xp3_option_candidates() {
            Ok(options) => options,
            Err(error) => {
                self.status = error;
                return;
            }
        };

        let Some(report) = &self.krkr_report else {
            return;
        };
        let output_root = PathBuf::from(clean_path_input(&self.krkr_output_path));
        if output_root.as_os_str().is_empty() {
            self.status = "Enter an output folder for the converted Suzu project.".to_owned();
            return;
        }
        let root = report.root.clone();
        let summary =
            match convert_krkr_package_to_suzu_project(&root, &output_root, &option_candidates) {
                Ok(summary) => summary,
                Err(error) => {
                    self.status = format!("Failed to convert KRKR startup flow: {error:#}");
                    return;
                }
            };

        self.project_path = output_root.display().to_string();
        self.scan_project();
        self.status = format!(
            "Converted KRKR startup flow -> {} ({} scripts, {} unreadable, {} lines, {} commands, {} choices).",
            summary.script_path.display(),
            summary.scripts,
            summary.unreadable,
            summary.lines,
            summary.commands,
            summary.choices
        );
    }

    fn xp3_options(&self) -> Result<Xp3Options, String> {
        let module_path = clean_path_input(&self.xp3_plugin_path);
        if module_path.is_empty() {
            return Ok(Xp3Options::default());
        }
        self.ensure_xp3_plugin_authorized()?;
        let module = Xp3PluginModule::from_json_file(&module_path)
            .map_err(|error| format!("Failed to load XP3 plugin module: {error:#}"))?;
        Ok(module.xp3_options())
    }

    fn xp3_option_candidates(&self) -> Result<Vec<Xp3Options>, String> {
        self.xp3_options().map(|options| vec![options])
    }

    fn xp3_plugin_requires_authorization(&self) -> bool {
        !clean_path_input(&self.xp3_plugin_path).is_empty()
    }

    fn ensure_xp3_plugin_authorized(&self) -> Result<(), String> {
        if self.xp3_plugin_requires_authorization() && !self.xp3_plugin_authorized {
            return Err(XP3_PLUGIN_AUTHORIZATION_MESSAGE.to_owned());
        }
        Ok(())
    }

    fn xp3_plugin_authorization_ui(&mut self, ui: &mut egui::Ui) {
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

    fn start_project_script(&mut self) {
        let Some(project) = &self.project else {
            self.status = "Open a project first.".to_owned();
            return;
        };
        let Some(index) = self.selected_project_script else {
            self.status = "Select a project script first.".to_owned();
            return;
        };
        let Some(script) = project.scripts.get(index) else {
            self.status = "Selected script no longer exists.".to_owned();
            return;
        };

        let script_path = project.root.join(script);
        let source = match fs::read_to_string(&script_path) {
            Ok(source) => source,
            Err(error) => {
                self.status = format!("Failed to read script: {error}");
                return;
            }
        };

        let mut app = preview_app("Project Preview");
        let _ = app.register_textures_from_dir(&project.root);
        if let Err(error) = app.load_script(&source) {
            self.status = format!("Failed to compile script: {error}");
            return;
        }
        app.advance_until_waiting();
        self.status = format!("Started project script {}.", script.display());
        self.game = Some(GamePreview::new(app, script.display().to_string()));
    }

    fn start_xp3_script(&mut self) {
        let path = match xp3_path_from_input(&self.xp3_path) {
            Ok(path) => path,
            Err(error) => {
                self.status = error;
                return;
            }
        };
        let Some(index) = self.selected_xp3_script else {
            self.status = "Select a script entry from the XP3 first.".to_owned();
            return;
        };
        let Some(entry) = self.xp3_entries.get(index) else {
            self.status = "Selected XP3 script no longer exists.".to_owned();
            return;
        };
        let script_id = asset_id_from_path(&entry.name);
        let options = match self.xp3_options() {
            Ok(options) => options,
            Err(error) => {
                self.status = error;
                return;
            }
        };

        let mut app = preview_app("XP3 Preview");
        match app
            .register_xp3_file_with_options(&path, options)
            .and_then(|_| app.load_script_asset(script_id.as_str()))
        {
            Ok(()) => {
                app.advance_until_waiting();
                self.status = format!("Started XP3 script `{script_id}`.");
                self.game = Some(GamePreview::new(app, entry.name.clone()));
            }
            Err(error) => {
                self.status = format!("Failed to start XP3 script: {error:#}");
            }
        }
    }

    fn load_dropped_path(&mut self, ctx: &egui::Context) {
        let dropped =
            ctx.input(|input| input.raw.dropped_files.iter().find_map(|f| f.path.clone()));
        let Some(path) = dropped else {
            return;
        };
        if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("xp3"))
        {
            self.xp3_path = path.display().to_string();
            self.load_xp3();
        } else if path.is_dir() {
            self.project_path = path.display().to_string();
            self.krkr_path = path.display().to_string();
            self.scan_project();
        }
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Project Suzu Launcher");
            ui.separator();
            ui.label(&self.status);
        });
    }

    fn project_panel(&mut self, ui: &mut egui::Ui) {
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

    fn xp3_panel(&mut self, ui: &mut egui::Ui) {
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

    fn game_panel(&mut self, ui: &mut egui::Ui) {
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

impl From<&Xp3Entry> for EntryRow {
    fn from(entry: &Xp3Entry) -> Self {
        Self {
            name: entry.name.clone(),
            kind: asset_type_from_path(&entry.name),
            protected: entry.protected,
            original_size: entry.original_size,
        }
    }
}
