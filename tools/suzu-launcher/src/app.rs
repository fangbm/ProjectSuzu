use std::{fs, path::PathBuf, process::Command};

use eframe::egui;
use suzu_asset::{
    probe_krkr_directory, AssetType, KrkrCompatibilityReport, Xp3Archive, Xp3Entry, Xp3Options,
    Xp3PluginModule,
};
use suzu_editor_core::ProjectIndex;
use suzu_project::{
    check_project, load_project, write_default_project_config, ProjectCheck, ProjectLoadOptions,
};

use crate::cli::XP3_PLUGIN_AUTHORIZATION_MESSAGE;
use crate::conversion::{convert_krkr_package_to_suzu_project, krkr_entry_looks_like_entrypoint};
use crate::paths::{
    asset_id_from_path, asset_type_from_path, clean_path_input, default_krkr_output_path,
    xp3_path_from_input,
};
use crate::preview::{install_cjk_fonts, preview_app, GamePreview};

pub(crate) struct LauncherApp {
    pub(crate) project_path: String,
    pub(crate) project: Option<ProjectIndex>,
    pub(crate) project_check: Option<ProjectCheck>,
    pub(crate) selected_project_script: Option<usize>,
    pub(crate) xp3_path: String,
    pub(crate) krkr_path: String,
    pub(crate) krkr_output_path: String,
    pub(crate) xp3_plugin_path: String,
    pub(crate) xp3_plugin_authorized: bool,
    pub(crate) krkr_report: Option<KrkrPackageReport>,
    pub(crate) krkr_compatibility: Option<KrkrCompatibilityReport>,
    pub(crate) xp3_entries: Vec<EntryRow>,
    pub(crate) selected_xp3_script: Option<usize>,
    pub(crate) game: Option<GamePreview>,
    pub(crate) status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct EntryRow {
    pub(crate) name: String,
    pub(crate) kind: AssetType,
    pub(crate) protected: bool,
    pub(crate) original_size: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct KrkrPackageReport {
    pub(crate) root: PathBuf,
    pub(crate) archives: Vec<KrkrArchiveSummary>,
    pub(crate) total_entries: usize,
    pub(crate) total_scripts: usize,
    pub(crate) protected_scripts: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct KrkrArchiveSummary {
    pub(crate) path: PathBuf,
    pub(crate) bytes: u64,
    pub(crate) entries: usize,
    pub(crate) scripts: usize,
    pub(crate) protected_scripts: usize,
    pub(crate) candidates: Vec<String>,
    pub(crate) error: Option<String>,
}

impl LauncherApp {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>, initial: PathBuf) -> Self {
        install_cjk_fonts(&cc.egui_ctx);
        let mut app = Self {
            project_path: String::new(),
            project: None,
            project_check: None,
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

    pub(crate) fn scan_project(&mut self) {
        let path = clean_path_input(&self.project_path);
        match ProjectIndex::scan(&path) {
            Ok(project) => {
                self.project_check =
                    match check_project(&project.root, ProjectLoadOptions::default()) {
                        Ok(report) => Some(report),
                        Err(error) => {
                            self.status =
                                format!("Project scanned, but standard check failed: {error:#}");
                            None
                        }
                    };
                if let Some(report) = &self.project_check {
                    self.status = format!(
                        "Project ready: {} scripts, {} resources, entry {}.",
                        project.scripts.len(),
                        project.resources.len(),
                        report.entry_path.display()
                    );
                }
                self.project_path = project.root.display().to_string();
                self.selected_project_script = self
                    .project_check
                    .as_ref()
                    .and_then(|report| {
                        report
                            .entry_path
                            .strip_prefix(&project.root)
                            .ok()
                            .and_then(|entry| {
                                project.scripts.iter().position(|script| script == entry)
                            })
                    })
                    .or_else(|| (!project.scripts.is_empty()).then_some(0));
                self.project = Some(project);
                self.game = None;
            }
            Err(error) => {
                self.project = None;
                self.project_check = None;
                self.selected_project_script = None;
                self.status = format!("Failed to open project: {error:#}");
            }
        }
    }

    pub(crate) fn check_project(&mut self) {
        let path = PathBuf::from(clean_path_input(&self.project_path));
        match check_project(&path, ProjectLoadOptions::default()) {
            Ok(report) => {
                let warnings = report.warnings();
                let warning_note = if warnings.is_empty() {
                    String::new()
                } else {
                    format!(" {} warnings.", warnings.len())
                };
                self.status = format!(
                    "Project check OK: {} assets, {} packages, entry {}.{}",
                    report.registered_assets,
                    report.registered_packages,
                    report.entry_path.display(),
                    warning_note
                );
                self.project_check = Some(report);
            }
            Err(error) => {
                self.project_check = None;
                self.status = format!("Project check failed: {error:#}");
            }
        }
    }

    pub(crate) fn create_project_config(&mut self) {
        let path = PathBuf::from(clean_path_input(&self.project_path));
        match write_default_project_config(&path) {
            Ok(config_path) => {
                self.status = format!("Created {}.", config_path.display());
                self.scan_project();
            }
            Err(error) => {
                self.status = format!("Failed to create project config: {error:#}");
            }
        }
    }

    pub(crate) fn open_project_in_editor(&mut self) {
        let project_path = PathBuf::from(clean_path_input(&self.project_path));
        if project_path.as_os_str().is_empty() {
            self.status = "Open a project folder first.".to_owned();
            return;
        }
        let editor = std::env::current_exe()
            .map(|mut path| {
                path.set_file_name(format!("suzu-editor{}", std::env::consts::EXE_SUFFIX));
                path
            })
            .unwrap_or_else(|_| PathBuf::from("suzu-editor"));
        match Command::new(&editor).arg(&project_path).spawn() {
            Ok(_) => {
                self.status = format!("Opened editor for {}.", project_path.display());
            }
            Err(error) => {
                self.status = format!("Failed to open editor {}: {error}", editor.display());
            }
        }
    }

    pub(crate) fn load_xp3(&mut self) {
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

    pub(crate) fn scan_krkr_package(&mut self) {
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

    pub(crate) fn convert_first_readable_krkr_script(&mut self) {
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

    pub(crate) fn xp3_options(&self) -> Result<Xp3Options, String> {
        let module_path = clean_path_input(&self.xp3_plugin_path);
        if module_path.is_empty() {
            return Ok(Xp3Options::default());
        }
        self.ensure_xp3_plugin_authorized()?;
        let module = Xp3PluginModule::from_json_file(&module_path)
            .map_err(|error| format!("Failed to load XP3 plugin module: {error:#}"))?;
        Ok(module.xp3_options())
    }

    pub(crate) fn xp3_option_candidates(&self) -> Result<Vec<Xp3Options>, String> {
        self.xp3_options().map(|options| vec![options])
    }

    pub(crate) fn xp3_plugin_requires_authorization(&self) -> bool {
        !clean_path_input(&self.xp3_plugin_path).is_empty()
    }

    pub(crate) fn ensure_xp3_plugin_authorized(&self) -> Result<(), String> {
        if self.xp3_plugin_requires_authorization() && !self.xp3_plugin_authorized {
            return Err(XP3_PLUGIN_AUTHORIZATION_MESSAGE.to_owned());
        }
        Ok(())
    }

    pub(crate) fn start_project_script(&mut self) {
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

        let loaded = match load_project(
            &project.root,
            ProjectLoadOptions {
                entry_override: Some(script.clone()),
            },
        ) {
            Ok(loaded) => loaded,
            Err(error) => {
                self.status = format!("Failed to start project script: {error:#}");
                return;
            }
        };
        self.status = format!("Started project script {}.", script.display());
        self.game = Some(GamePreview::new(
            loaded.app,
            loaded.entry_path.display().to_string(),
        ));
    }

    pub(crate) fn start_project_entry(&mut self) {
        let path = PathBuf::from(clean_path_input(&self.project_path));
        let loaded = match load_project(&path, ProjectLoadOptions::default()) {
            Ok(loaded) => loaded,
            Err(error) => {
                self.status = format!("Failed to start project: {error:#}");
                return;
            }
        };
        self.status = format!("Started project entry {}.", loaded.entry_path.display());
        self.project_check = Some(ProjectCheck::from_loaded(&loaded));
        self.game = Some(GamePreview::new(
            loaded.app,
            loaded.entry_path.display().to_string(),
        ));
    }

    pub(crate) fn start_xp3_script(&mut self) {
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

    pub(crate) fn load_dropped_path(&mut self, ctx: &egui::Context) {
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
