#![cfg_attr(windows, windows_subsystem = "windows")]

use std::{
    collections::{HashMap, HashSet, VecDeque},
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{bail, Context};
use eframe::egui;
use encoding_rs::{SHIFT_JIS, UTF_16BE, UTF_16LE};
use suzu_app::{GameConfig, SuzuApp, TitleScreenConfig};
use suzu_asset::{AssetType, Xp3Archive, Xp3Decryptor, Xp3Entry, Xp3Options};
use suzu_editor_core::{convert_krkr_ks_to_szs, ProjectIndex};
use suzu_platform::{DesktopApp, DesktopFrame, DesktopInputEvent, FrameSprite, FrameText};
use suzu_script::CURRENT_SCRIPT_FORMAT_VERSION;

fn main() -> eframe::Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args
        .first()
        .and_then(|arg| arg.to_str())
        .is_some_and(|arg| arg == "--krkr2suzu")
    {
        if let Err(error) = run_krkr2suzu_cli(&args[1..]) {
            eprintln!("krkr2suzu failed: {error:#}");
        }
        return Ok(());
    }

    let initial = args.first().map(PathBuf::from).unwrap_or_default();
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

fn run_krkr2suzu_cli(args: &[OsString]) -> anyhow::Result<()> {
    if args.len() < 2 {
        bail!("usage: suzu-launcher --krkr2suzu <krkr-folder> <output-folder> [--xor <hex-byte>]");
    }
    let root = PathBuf::from(&args[0]);
    let output = PathBuf::from(&args[1]);
    let mut options = vec![Xp3Options::default()];
    if args.len() >= 4 && args[2].to_string_lossy() == "--xor" {
        if args[3].to_string_lossy().eq_ignore_ascii_case("auto") {
            options = auto_xor_option_candidates();
        } else {
            let key = parse_xor_key(&args[3].to_string_lossy())?;
            options = xor_option_candidates_for_key(key);
        }
    }

    let summary = convert_krkr_package_to_suzu_project(&root, &output, &options)?;
    println!(
        "Converted {} scripts ({} unreadable) to {} from {} lines, {} commands, {} choices.",
        summary.scripts,
        summary.unreadable,
        summary.script_path.display(),
        summary.lines,
        summary.commands,
        summary.choices
    );
    Ok(())
}

#[derive(Debug, Clone)]
struct KrkrConversionSummary {
    script_path: PathBuf,
    scripts: usize,
    unreadable: usize,
    lines: usize,
    commands: usize,
    choices: usize,
}

fn convert_krkr_package_to_suzu_project(
    root: &Path,
    output_root: &Path,
    option_candidates: &[Xp3Options],
) -> anyhow::Result<KrkrConversionSummary> {
    let script_dir = output_root.join("script");
    fs::create_dir_all(&script_dir)
        .with_context(|| format!("failed to create {}", script_dir.display()))?;

    let mut archive_candidates = Vec::<Vec<Xp3Archive>>::new();
    let mut script_locations = HashMap::<String, (usize, String)>::new();
    let mut entrypoints = Vec::<String>::new();
    let mut fallback_scripts = Vec::<String>::new();

    for entry in fs::read_dir(root).with_context(|| format!("failed to scan {}", root.display()))? {
        let path = entry?.path();
        if !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("xp3"))
        {
            continue;
        }
        let base_archive = match Xp3Archive::from_file_with_options(&path, Xp3Options::default()) {
            Ok(archive) => archive,
            Err(_) => continue,
        };
        let candidates = option_candidates
            .iter()
            .map(|options| base_archive.clone().with_options(options.clone()))
            .collect::<Vec<_>>();
        let Some(first_archive) = candidates.first() else {
            continue;
        };
        let archive_index = archive_candidates.len();
        let mut entries = first_archive
            .entries()
            .iter()
            .filter(|entry| script_extension_is(&entry.name, "ks"))
            .map(|entry| entry.name.clone())
            .collect::<Vec<_>>();
        entries.sort_by_key(|name| {
            (
                !krkr_entry_looks_like_entrypoint(name),
                name.to_ascii_lowercase(),
            )
        });

        for name in entries {
            for alias in krkr_script_lookup_keys(&name) {
                script_locations.insert(alias, (archive_index, name.clone()));
            }
            if krkr_entry_looks_like_entrypoint(&name) {
                entrypoints.push(name.clone());
            }
            fallback_scripts.push(name);
        }
        archive_candidates.push(candidates);
    }

    if script_locations.is_empty() {
        bail!("no .ks scripts found");
    }

    entrypoints.sort_by_key(|name| {
        (
            !krkr_entry_looks_like_entrypoint(name),
            name.to_ascii_lowercase(),
        )
    });
    let roots = if entrypoints.is_empty() {
        fallback_scripts.into_iter().take(1).collect::<Vec<_>>()
    } else {
        entrypoints
    };

    let mut scripts = Vec::<(String, Vec<u8>)>::new();
    let mut unreadable = 0usize;
    let mut queue = roots.into_iter().collect::<VecDeque<_>>();
    let mut visited = HashSet::<String>::new();
    while let Some(script_name) = queue.pop_front() {
        let lookup_key = normalize_krkr_script_key(&script_name);
        if !visited.insert(lookup_key.clone()) {
            continue;
        }
        if visited.len() > 512 {
            break;
        }

        let Some((archive_index, entry_name)) = script_locations.get(&lookup_key) else {
            continue;
        };
        let Some(bytes) = read_best_krkr_script(&archive_candidates[*archive_index], entry_name)
        else {
            unreadable += 1;
            continue;
        };

        let source = decode_krkr_text(&bytes);
        for reference in krkr_script_references(&source, entry_name) {
            if let Some((_, resolved_name)) =
                script_locations.get(&normalize_krkr_script_key(&reference))
            {
                queue.push_back(resolved_name.clone());
            }
        }
        scripts.push((entry_name.clone(), bytes));
    }

    if scripts.is_empty() {
        bail!("no readable .ks scripts found");
    }

    scripts.sort_by_key(|(name, _)| {
        (
            !krkr_entry_looks_like_entrypoint(name),
            name.to_ascii_lowercase(),
        )
    });
    scripts.dedup_by(|(left, _), (right, _)| left.eq_ignore_ascii_case(right));

    let mut output = format!(
        "@script version={CURRENT_SCRIPT_FORMAT_VERSION}\n; Converted from KRKR/KAG package: {}\n",
        root.display()
    );
    let mut total_lines = 0usize;
    let mut total_commands = 0usize;
    let mut total_choices = 0usize;
    for (entry_name, bytes) in &scripts {
        let source = decode_krkr_text(bytes);
        let converted = convert_krkr_ks_to_szs(&source, Some(entry_name));
        total_lines += converted.report.lines_read;
        total_commands += converted.report.commands_converted;
        total_choices += converted.report.choices;
        output.push('\n');
        for label in krkr_script_labels(entry_name) {
            output.push_str(&format!("*{label}\n"));
        }
        for line in converted.source.lines() {
            if line.trim_start().starts_with("@script ") {
                continue;
            }
            output.push_str(line);
            output.push('\n');
        }
    }

    suzu_script::compile_script(&output).context("converted KRKR startup flow did not compile")?;
    let script_path = script_dir.join("main.szs");
    fs::write(&script_path, output)
        .with_context(|| format!("failed to write {}", script_path.display()))?;

    Ok(KrkrConversionSummary {
        script_path,
        scripts: scripts.len(),
        unreadable,
        lines: total_lines,
        commands: total_commands,
        choices: total_choices,
    })
}

struct LauncherApp {
    project_path: String,
    project: Option<ProjectIndex>,
    selected_project_script: Option<usize>,
    xp3_path: String,
    krkr_path: String,
    krkr_output_path: String,
    krkr_report: Option<KrkrPackageReport>,
    xor_enabled: bool,
    xor_key: String,
    xp3_entries: Vec<EntryRow>,
    selected_xp3_script: Option<usize>,
    game: Option<GamePreview>,
    status: String,
}

#[derive(Debug, Clone)]
struct EntryRow {
    name: String,
    kind: AssetType,
    encrypted: bool,
    original_size: u64,
}

#[derive(Debug, Clone)]
struct KrkrPackageReport {
    root: PathBuf,
    archives: Vec<KrkrArchiveSummary>,
    total_entries: usize,
    total_scripts: usize,
    encrypted_scripts: usize,
}

#[derive(Debug, Clone)]
struct KrkrArchiveSummary {
    path: PathBuf,
    bytes: u64,
    entries: usize,
    scripts: usize,
    encrypted_scripts: usize,
    candidates: Vec<String>,
    error: Option<String>,
}

struct GamePreview {
    app: SuzuApp,
    label: String,
    textures: HashMap<String, egui::TextureHandle>,
    last_frame: Instant,
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
            krkr_report: None,
            xor_enabled: false,
            xor_key: "5A".to_owned(),
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
                self.status = error;
                return;
            }
        };
        let mut archives = Vec::new();
        let mut total_entries = 0;
        let mut total_scripts = 0;
        let mut encrypted_scripts = 0;

        let read_dir = match fs::read_dir(&root) {
            Ok(read_dir) => read_dir,
            Err(error) => {
                self.krkr_report = None;
                self.status = format!("Failed to scan KRKR folder: {error}");
                return;
            }
        };

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
                    let encrypted = scripts.iter().filter(|row| row.encrypted).count();
                    let candidates = scripts
                        .iter()
                        .filter(|row| krkr_entry_looks_like_entrypoint(&row.name))
                        .take(12)
                        .map(|row| row.name.clone())
                        .collect::<Vec<_>>();
                    total_entries += rows.len();
                    total_scripts += scripts.len();
                    encrypted_scripts += encrypted;
                    KrkrArchiveSummary {
                        path,
                        bytes,
                        entries: rows.len(),
                        scripts: scripts.len(),
                        encrypted_scripts: encrypted,
                        candidates,
                        error: None,
                    }
                }
                Err(error) => KrkrArchiveSummary {
                    path,
                    bytes,
                    entries: 0,
                    scripts: 0,
                    encrypted_scripts: 0,
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
            encrypted_scripts,
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
        if !self.xor_enabled {
            return Ok(Xp3Options::default());
        }
        Ok(Xp3Options {
            decryptor: Xp3Decryptor::Xor {
                key: self.xor_key()?,
            },
        })
    }

    fn xp3_option_candidates(&self) -> Result<Vec<Xp3Options>, String> {
        if !self.xor_enabled {
            return Ok(vec![Xp3Options::default()]);
        }
        let key = self.xor_key()?;
        Ok(xor_option_candidates_for_key(key))
    }

    fn xor_key(&self) -> Result<u8, String> {
        parse_xor_key(&self.xor_key)
            .map_err(|_| "XOR key must be a byte, for example 5A or 90.".to_owned())
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
            ui.checkbox(&mut self.xor_enabled, "XOR encrypted segments");
            ui.add_enabled(
                self.xor_enabled,
                egui::TextEdit::singleline(&mut self.xor_key).desired_width(64.0),
            );
            if ui.button("Run Selected Script").clicked() {
                self.start_xp3_script();
            }
        });

        ui.separator();
        ui.label(format!("{} entries", self.xp3_entries.len()));
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (index, entry) in self.xp3_entries.iter().enumerate() {
                if !matches!(entry.kind, AssetType::Script | AssetType::Data) {
                    continue;
                }
                let lock = if entry.encrypted { "locked" } else { "plain" };
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
                "{} XP3 · {} entries · {} script-like · {} encrypted scripts",
                report.archives.len(),
                report.total_entries,
                report.total_scripts,
                report.encrypted_scripts
            ));
            ui.label(format!("Root: {}", report.root.display()));
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
                            "{name} · {} MB · {} entries · {} scripts · {} encrypted",
                            archive.bytes / 1024 / 1024,
                            archive.entries,
                            archive.scripts,
                            archive.encrypted_scripts
                        ));
                        for candidate in &archive.candidates {
                            ui.label(format!("  entry: {candidate}"));
                        }
                    }
                });
            if report.encrypted_scripts > 0 {
                if self.xor_enabled {
                    ui.colored_label(
                        egui::Color32::from_rgb(64, 128, 64),
                        "XOR decryptor is enabled for KRKR scan and conversion.",
                    );
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(190, 92, 32),
                        "KRKR scripts are encrypted; enable XOR or add a game-specific decryptor before conversion.",
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

impl GamePreview {
    fn new(app: SuzuApp, label: String) -> Self {
        Self {
            app,
            label,
            textures: HashMap::new(),
            last_frame: Instant::now(),
        }
    }
}

impl From<&Xp3Entry> for EntryRow {
    fn from(entry: &Xp3Entry) -> Self {
        Self {
            name: entry.name.clone(),
            kind: asset_type_from_path(&entry.name),
            encrypted: entry.encrypted,
            original_size: entry.original_size,
        }
    }
}

fn preview_app(subtitle: &str) -> SuzuApp {
    SuzuApp::new(GameConfig {
        title_screen: TitleScreenConfig {
            enabled: false,
            title: "Project Suzu".to_owned(),
            subtitle: subtitle.to_owned(),
        },
        ..GameConfig::default()
    })
}

fn asset_type_from_path(path: &str) -> AssetType {
    match Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png" | "jpg" | "jpeg" | "webp") => AssetType::Texture,
        Some("ogg" | "wav" | "mp3" | "flac") => AssetType::Audio,
        Some("szs" | "ks" | "tjs" | "txt") => AssetType::Script,
        Some("ttf" | "otf") => AssetType::Font,
        Some(_) => AssetType::Data,
        None => AssetType::Unknown,
    }
}

fn asset_id_from_path(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(path)
        .to_owned()
}

fn krkr_entry_looks_like_entrypoint(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "startup.tjs"
            | "system/startup.tjs"
            | "appconfig.tjs"
            | "main/config.tjs"
            | "main/envinit.tjs"
            | "main/default.tjs"
            | "main/custom.ks"
            | "main/custom.tjs"
            | "first.ks"
            | "start.ks"
            | "title.ks"
    ) || normalized.ends_with("/startup.tjs")
        || normalized.ends_with("/first.ks")
        || normalized.ends_with("/start.ks")
        || normalized.ends_with("/title.ks")
}

fn default_krkr_output_path(root: &Path) -> PathBuf {
    let folder_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(|name| format!("{name}-suzu-migration"))
        .unwrap_or_else(|| "suzu-migration".to_owned());
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .map(|home| {
            home.join("Documents")
                .join("ProjectSuzu Migrations")
                .join(&folder_name)
        })
        .unwrap_or_else(|| root.join(folder_name))
}

fn script_extension_is(path: &str, extension: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

fn read_best_krkr_script(archives: &[Xp3Archive], entry_name: &str) -> Option<Vec<u8>> {
    archives
        .iter()
        .filter_map(|archive| {
            let bytes = archive.read_file(entry_name).ok()?;
            let text = decode_krkr_text(&bytes);
            Some((score_krkr_text(&text), bytes))
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, bytes)| bytes)
}

fn score_krkr_text(text: &str) -> i64 {
    if text.contains("This is a protected archive") {
        return i64::MIN / 2;
    }
    let mut score = 0i64;
    for ch in text.chars() {
        match ch {
            '[' | ']' | '@' | '*' | '=' | ';' | '"' | '\'' => score += 8,
            '\n' | '\r' | '\t' => score += 2,
            '\u{20}'..='\u{7e}' => score += 3,
            '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{9fff}' => score += 2,
            '\u{0}'..='\u{8}' | '\u{b}' | '\u{c}' | '\u{e}'..='\u{1f}' => score -= 10,
            '\u{fffd}' => score -= 30,
            _ => score -= 1,
        }
    }
    score
}

fn krkr_script_references(source: &str, current_entry: &str) -> Vec<String> {
    let mut references = Vec::new();
    for token in source.split(|ch: char| ch.is_whitespace() || matches!(ch, '[' | ']')) {
        let Some((key, raw_value)) = token.split_once('=') else {
            continue;
        };
        if !key.eq_ignore_ascii_case("storage") {
            continue;
        }
        let value = raw_value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim_matches(';');
        if !script_extension_is(value, "ks") {
            continue;
        }
        push_unique_reference(&mut references, value);
        if let Some(parent) = Path::new(current_entry).parent() {
            let relative = parent.join(value).to_string_lossy().replace('\\', "/");
            push_unique_reference(&mut references, &relative);
        }
    }
    references
}

fn push_unique_reference(references: &mut Vec<String>, value: &str) {
    if !value.is_empty()
        && !references
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(value))
    {
        references.push(value.to_owned());
    }
}

fn krkr_script_lookup_keys(path: &str) -> Vec<String> {
    let mut keys = Vec::new();
    push_unique_lookup_key(&mut keys, path);
    if let Some(file_name) = Path::new(path).file_name().and_then(|value| value.to_str()) {
        push_unique_lookup_key(&mut keys, file_name);
    }
    keys
}

fn push_unique_lookup_key(keys: &mut Vec<String>, value: &str) {
    let key = normalize_krkr_script_key(value);
    if !key.is_empty() && !keys.iter().any(|existing| existing == &key) {
        keys.push(key);
    }
}

fn normalize_krkr_script_key(path: &str) -> String {
    path.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn krkr_script_labels(path: &str) -> Vec<String> {
    let mut labels = Vec::new();
    push_unique_label(&mut labels, path);
    if let Some(file_name) = Path::new(path).file_name().and_then(|value| value.to_str()) {
        push_unique_label(&mut labels, file_name);
    }
    if let Some(stem) = Path::new(path).file_stem().and_then(|value| value.to_str()) {
        push_unique_label(&mut labels, stem);
    }
    labels
}

fn push_unique_label(labels: &mut Vec<String>, raw: &str) {
    let label = sanitize_krkr_label(raw);
    if !label.is_empty() && !labels.iter().any(|existing| existing == &label) {
        labels.push(label);
    }
}

fn sanitize_krkr_label(label: &str) -> String {
    let label = label.trim().trim_start_matches('*');
    label
        .chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | '.' => ch,
            _ => '_',
        })
        .collect()
}

fn parse_xor_key(value: &str) -> anyhow::Result<u8> {
    let key_text = value.trim().trim_start_matches("0x");
    u8::from_str_radix(key_text, 16)
        .or_else(|_| value.trim().parse::<u8>())
        .with_context(|| format!("XOR key must be a byte, got `{value}`"))
}

fn xor_option_candidates_for_key(key: u8) -> Vec<Xp3Options> {
    vec![
        Xp3Options {
            decryptor: Xp3Decryptor::Xor { key },
        },
        Xp3Options {
            decryptor: Xp3Decryptor::XorAfterInflate { key },
        },
    ]
}

fn auto_xor_option_candidates() -> Vec<Xp3Options> {
    let mut options = Vec::with_capacity(513);
    options.push(Xp3Options::default());
    for key in 0u8..=u8::MAX {
        options.extend(xor_option_candidates_for_key(key));
    }
    options
}

fn decode_krkr_text(bytes: &[u8]) -> String {
    if let Some(rest) = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]) {
        return String::from_utf8_lossy(rest).into_owned();
    }
    if let Some(rest) = bytes.strip_prefix(&[0xff, 0xfe]) {
        let (text, _, _) = UTF_16LE.decode(rest);
        return text.into_owned();
    }
    if let Some(rest) = bytes.strip_prefix(&[0xfe, 0xff]) {
        let (text, _, _) = UTF_16BE.decode(rest);
        return text.into_owned();
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }
    let (text, _, _) = SHIFT_JIS.decode(bytes);
    text.into_owned()
}

fn xp3_path_from_input(input: &str) -> Result<PathBuf, String> {
    let cleaned = clean_path_input(input);
    if cleaned.is_empty() {
        return Err("Enter an XP3 path first.".to_owned());
    }
    let path = PathBuf::from(cleaned);
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xp3"))
    {
        Ok(path)
    } else {
        Err("The selected file is not an .xp3 archive.".to_owned())
    }
}

fn clean_path_input(input: &str) -> String {
    let mut value = input.trim().trim_matches(['"', '\'']).trim().to_owned();
    if let Some(rest) = value.strip_prefix("file:///") {
        value = rest.replace('/', "\\");
    } else if let Some(rest) = value.strip_prefix("file://") {
        value = rest.replace('/', "\\");
    }
    value
}

fn render_frame(
    painter: &egui::Painter,
    bounds: egui::Rect,
    frame: &DesktopFrame,
    textures: &mut HashMap<String, egui::TextureHandle>,
) {
    painter.rect_filled(bounds, 0.0, color32(frame.clear_color, 1.0));
    for texture in &frame.textures {
        textures.entry(texture.id.clone()).or_insert_with(|| {
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [texture.width as usize, texture.height as usize],
                &texture.rgba,
            );
            painter
                .ctx()
                .load_texture(texture.id.clone(), image, Default::default())
        });
    }

    let mut sprites = frame.sprites.iter().collect::<Vec<_>>();
    sprites.sort_by_key(|sprite| sprite.z_index);
    for sprite in sprites {
        paint_sprite(painter, bounds, sprite, textures);
    }
    let mut texts = frame.texts.iter().collect::<Vec<_>>();
    texts.sort_by_key(|text| text.z_index);
    for text in texts {
        paint_text(painter, bounds, text);
    }
}

fn paint_sprite(
    painter: &egui::Painter,
    bounds: egui::Rect,
    sprite: &FrameSprite,
    textures: &HashMap<String, egui::TextureHandle>,
) {
    let rect = map_rect(bounds, sprite.bounds);
    let tint = color32(sprite.tint, sprite.opacity);
    if let Some(texture) = textures.get(&sprite.texture_id) {
        painter.image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            tint,
        );
    } else {
        painter.rect_filled(rect, 4.0, tint);
    }
}

fn paint_text(painter: &egui::Painter, bounds: egui::Rect, text: &FrameText) {
    let rect = map_rect(bounds, text.bounds);
    painter.text(
        rect.min,
        egui::Align2::LEFT_TOP,
        &text.content,
        egui::FontId::proportional(20.0),
        color32(text.color, 1.0),
    );
}

fn map_rect(bounds: egui::Rect, rect: suzu_core::Rect) -> egui::Rect {
    let scale_x = bounds.width() / 1280.0;
    let scale_y = bounds.height() / 720.0;
    egui::Rect::from_min_size(
        egui::pos2(
            bounds.left() + rect.origin.x * scale_x,
            bounds.top() + rect.origin.y * scale_y,
        ),
        egui::vec2(rect.size.x * scale_x, rect.size.y * scale_y),
    )
}

fn fit_size(size: egui::Vec2, bounds: egui::Vec2) -> egui::Vec2 {
    let scale = (bounds.x / size.x).min(bounds.y / size.y).min(1.0);
    size * scale.max(0.01)
}

fn color32(color: suzu_core::Color, opacity: f32) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (color.r.clamp(0.0, 1.0) * 255.0) as u8,
        (color.g.clamp(0.0, 1.0) * 255.0) as u8,
        (color.b.clamp(0.0, 1.0) * 255.0) as u8,
        ((color.a * opacity).clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn install_cjk_fonts(ctx: &egui::Context) {
    let Some((name, bytes)) = load_cjk_font() else {
        return;
    };
    let mut fonts = egui::FontDefinitions::default();
    fonts
        .font_data
        .insert(name.clone(), egui::FontData::from_owned(bytes));
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, name.clone());
    }
    ctx.set_fonts(fonts);
}

fn load_cjk_font() -> Option<(String, Vec<u8>)> {
    for path in cjk_font_candidates() {
        if let Ok(bytes) = fs::read(path) {
            return Some((format!("cjk-{}", Path::new(path).display()), bytes));
        }
    }
    None
}

fn cjk_font_candidates() -> &'static [&'static str] {
    &[
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyh.ttf",
        r"C:\Windows\Fonts\meiryo.ttc",
        r"C:\Windows\Fonts\YuGothM.ttc",
        r"C:\Windows\Fonts\msgothic.ttc",
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/System/Library/Fonts/PingFang.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_quoted_path() {
        assert_eq!(
            clean_path_input(r#""D:\games\Suzu\data.xp3""#),
            r"D:\games\Suzu\data.xp3"
        );
    }

    #[test]
    fn recognizes_xp3_paths() {
        assert!(xp3_path_from_input(r"D:\games\Suzu\data.xp3").is_ok());
        assert!(xp3_path_from_input(r"D:\games\Suzu\data.zip").is_err());
    }
}
