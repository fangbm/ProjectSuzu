#![cfg_attr(windows, windows_subsystem = "windows")]

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use eframe::egui;
use suzu_editor_core::{
    analyze_graph, export_szs, import_szs, Diagnostic, DiagnosticLevel, EditorDocument,
    EditorNodeKind, ProjectIndex,
};

fn main() -> eframe::Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    match dispatch_cli(&args) {
        Ok(CliAction::Handled) => return Ok(()),
        Ok(CliAction::LaunchGui) => {}
        Err(error) => {
            if args
                .first()
                .and_then(|arg| arg.to_str())
                .is_some_and(|arg| arg == "--check")
            {
                eprintln!("Project Suzu Editor check FAILED");
                eprintln!("reason: {error:#}");
            } else {
                eprintln!("{error:#}");
            }
            std::process::exit(1);
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 760.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Project Suzu Editor",
        options,
        Box::new(|_cc| Box::<EditorApp>::default()),
    )
}

enum CliAction {
    Handled,
    LaunchGui,
}

fn dispatch_cli(args: &[OsString]) -> anyhow::Result<CliAction> {
    if args
        .first()
        .and_then(|arg| arg.to_str())
        .is_some_and(|arg| arg == "--check")
    {
        run_check_cli(&args[1..]).context("editor check failed")?;
        return Ok(CliAction::Handled);
    }

    Ok(CliAction::LaunchGui)
}

fn run_check_cli(args: &[OsString]) -> anyhow::Result<()> {
    let mut project_root = std::env::current_dir()?;
    let mut index = 0;
    while index < args.len() {
        match args[index].to_string_lossy().as_ref() {
            "--project-root" if index + 1 < args.len() => {
                project_root = PathBuf::from(clean_path_input(&args[index + 1].to_string_lossy()));
                index += 2;
            }
            "--project-root" => anyhow::bail!("--project-root requires a folder path"),
            other => anyhow::bail!("unknown check option `{other}`"),
        }
    }

    if !project_root.exists() {
        anyhow::bail!("project root does not exist: {}", project_root.display());
    }
    ProjectIndex::scan(&project_root)?;
    println!("Project Suzu Editor check OK");
    println!("version: {}", env!("CARGO_PKG_VERSION"));
    println!("features: project-scan, visual-script, gui");
    Ok(())
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

struct EditorApp {
    project_root: String,
    project: Option<ProjectIndex>,
    document: EditorDocument,
    selected_node: Option<usize>,
    diagnostics: Vec<Diagnostic>,
    status: String,
}

impl Default for EditorApp {
    fn default() -> Self {
        let project_root = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .display()
            .to_string();
        let mut app = Self {
            project_root,
            project: None,
            document: EditorDocument::default(),
            selected_node: None,
            diagnostics: Vec::new(),
            status: "Open a Project Suzu folder to begin.".to_owned(),
        };
        app.scan_project();
        app
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| self.menu_bar(ui));
        egui::SidePanel::left("project")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| self.project_panel(ui));
        egui::SidePanel::right("inspector")
            .resizable(true)
            .default_width(340.0)
            .show(ctx, |ui| self.inspector_panel(ui));
        egui::TopBottomPanel::bottom("diagnostics")
            .resizable(true)
            .default_height(130.0)
            .show(ctx, |ui| self.diagnostics_panel(ui));
        egui::CentralPanel::default().show(ctx, |ui| self.node_panel(ui));
    }
}

impl EditorApp {
    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Project Suzu Editor");
            ui.separator();
            if ui.button("Scan").clicked() {
                self.scan_project();
            }
            if ui.button("New").clicked() {
                self.new_document();
            }
            if ui.button("Export").clicked() {
                self.export_document();
            }
            if ui.button("Compile Check").clicked() {
                self.refresh_diagnostics();
            }
            ui.separator();
            ui.label(&self.status);
        });
    }

    fn project_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Project");
        ui.label("Root");
        ui.text_edit_singleline(&mut self.project_root);
        if ui.button("Open Folder").clicked() {
            self.scan_project();
        }

        ui.separator();
        ui.heading("Scripts");
        if let Some(project) = &self.project {
            let root = project.root.clone();
            let scripts = project.scripts.clone();
            let resources = project.resources.clone();
            for script in scripts {
                if ui.button(script.display().to_string()).clicked() {
                    self.open_script(&root.join(script));
                }
            }

            ui.separator();
            ui.heading("Assets");
            for resource in resources.iter().take(48) {
                ui.label(format!("{:?}: {}", resource.kind, resource.path.display()));
            }
            if resources.len() > 48 {
                ui.label(format!("... {} more", resources.len() - 48));
            }
        } else {
            ui.label("No project loaded.");
        }
    }

    fn node_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Script Nodes");
        ui.horizontal(|ui| {
            if ui.button("+ Dialogue").clicked() {
                self.push_node(EditorNodeKind::Dialogue {
                    speaker: Some("Narrator".to_owned()),
                    text: "New line".to_owned(),
                });
            }
            if ui.button("+ Background").clicked() {
                self.push_node(EditorNodeKind::Background {
                    file: "background".to_owned(),
                    method: suzu_editor_core::TransitionForm::CrossFade { duration_ms: 500 },
                    time_ms: 500,
                });
            }
            if ui.button("+ Choice").clicked() {
                self.push_node(EditorNodeKind::Choice {
                    options: vec![suzu_editor_core::ChoiceOptionForm {
                        text: "Choice".to_owned(),
                        goto: "next".to_owned(),
                        condition: None,
                    }],
                });
            }
            if ui.button("+ Label").clicked() {
                self.push_node(EditorNodeKind::Label {
                    name: "next".to_owned(),
                });
            }
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (index, node) in self.document.nodes.iter().enumerate() {
                let selected = self.selected_node == Some(index);
                let label = format!("{:03}  {}", index + 1, node.title);
                if ui.selectable_label(selected, label).clicked() {
                    self.selected_node = Some(index);
                }
            }
        });
    }

    fn inspector_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Inspector");
        let Some(index) = self.selected_node else {
            ui.label("Select a node to edit.");
            return;
        };
        let Some(node) = self.document.nodes.get_mut(index) else {
            self.selected_node = None;
            return;
        };

        ui.label(format!("Node {:?}", node.id));
        ui.text_edit_singleline(&mut node.title);
        ui.separator();

        match &mut node.kind {
            EditorNodeKind::ScriptHeader { version } => {
                ui.label("Script version");
                ui.add(egui::DragValue::new(version).clamp_range(1..=99));
            }
            EditorNodeKind::Label { name } => {
                ui.label("Label");
                ui.text_edit_singleline(name);
            }
            EditorNodeKind::Dialogue { speaker, text } => {
                let mut has_speaker = speaker.is_some();
                ui.checkbox(&mut has_speaker, "Speaker");
                if has_speaker {
                    ui.text_edit_singleline(speaker.get_or_insert_with(String::new));
                } else {
                    *speaker = None;
                }
                ui.label("Text");
                ui.text_edit_multiline(text);
            }
            EditorNodeKind::Background { file, time_ms, .. } => {
                ui.label("Background asset id");
                ui.text_edit_singleline(file);
                ui.label("Time ms");
                ui.add(egui::DragValue::new(time_ms).clamp_range(0..=60_000));
            }
            EditorNodeKind::Character {
                name,
                face,
                layer,
                flip,
                ..
            } => {
                ui.label("Character name");
                ui.text_edit_singleline(name);
                ui.label("Face");
                ui.text_edit_singleline(face.get_or_insert_with(String::new));
                ui.add(egui::DragValue::new(layer));
                ui.checkbox(flip, "Flip horizontally");
            }
            EditorNodeKind::Choice { options } => {
                for (option_index, option) in options.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.label(format!("Option {}", option_index + 1));
                        ui.text_edit_singleline(&mut option.text);
                        ui.label("Goto");
                        ui.text_edit_singleline(&mut option.goto);
                        ui.label("Condition");
                        ui.text_edit_singleline(option.condition.get_or_insert_with(String::new));
                    });
                }
                if ui.button("Add option").clicked() {
                    options.push(suzu_editor_core::ChoiceOptionForm {
                        text: "New option".to_owned(),
                        goto: String::new(),
                        condition: None,
                    });
                }
            }
            EditorNodeKind::Jump { label } | EditorNodeKind::Call { label } => {
                ui.label("Target label");
                ui.text_edit_singleline(label);
            }
            EditorNodeKind::SetVariable { name, value } => {
                ui.label("Variable");
                ui.text_edit_singleline(name);
                ui.label("Value");
                ui.text_edit_singleline(value);
            }
            EditorNodeKind::Wait { time_ms } => {
                ui.add(egui::DragValue::new(time_ms).clamp_range(0..=60_000));
            }
            EditorNodeKind::SaveName { text } | EditorNodeKind::RawText { source: text } => {
                ui.text_edit_multiline(text);
            }
            EditorNodeKind::MessageBox { visible } => {
                ui.checkbox(visible, "Visible");
            }
            _ => {
                ui.label("This node type is currently shown in compact mode.");
                ui.monospace(format!("{:#?}", node.kind));
            }
        }

        ui.separator();
        if ui.button("Delete Node").clicked() {
            self.document.nodes.remove(index);
            self.selected_node = None;
            self.refresh_diagnostics();
        }
    }

    fn diagnostics_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Diagnostics");
            if ui.button("Refresh").clicked() {
                self.refresh_diagnostics();
            }
        });
        egui::ScrollArea::vertical().show(ui, |ui| {
            for diagnostic in &self.diagnostics {
                let level = match diagnostic.level {
                    DiagnosticLevel::Error => "error",
                    DiagnosticLevel::Warning => "warning",
                    DiagnosticLevel::Info => "info",
                };
                ui.label(format!("{level}: {}", diagnostic.message));
            }
        });
    }

    fn scan_project(&mut self) {
        match ProjectIndex::scan(Path::new(&self.project_root)) {
            Ok(project) => {
                self.status = format!(
                    "Loaded {} scripts and {} resources.",
                    project.scripts.len(),
                    project.resources.len()
                );
                self.project = Some(project);
            }
            Err(error) => self.status = format!("Project scan failed: {error:#}"),
        }
    }

    fn new_document(&mut self) {
        let mut document = EditorDocument::default();
        document.push_node(
            EditorNodeKind::ScriptHeader {
                version: suzu_script_version(),
            },
            None,
        );
        document.push_node(
            EditorNodeKind::Dialogue {
                speaker: Some("Narrator".to_owned()),
                text: "Hello from Project Suzu.".to_owned(),
            },
            None,
        );
        self.document = document;
        self.selected_node = Some(0);
        self.refresh_diagnostics();
        self.status = "New document created.".to_owned();
    }

    fn open_script(&mut self, path: &Path) {
        match fs::read_to_string(path) {
            Ok(source) => {
                self.document = import_szs(&source, Some(path.to_path_buf()));
                self.selected_node = self.document.nodes.first().map(|_| 0);
                self.refresh_diagnostics();
                self.status = format!("Opened {}", path.display());
            }
            Err(error) => self.status = format!("Open failed: {error}"),
        }
    }

    fn export_document(&mut self) {
        match export_szs(&self.document) {
            Ok(source) => {
                if let Some(path) = &self.document.source_path {
                    match fs::write(path, source) {
                        Ok(()) => self.status = format!("Saved {}", path.display()),
                        Err(error) => self.status = format!("Save failed: {error}"),
                    }
                } else {
                    self.status = "Export succeeded; no source path is attached.".to_owned();
                }
            }
            Err(error) => self.status = format!("Export failed: {error:#}"),
        }
        self.refresh_diagnostics();
    }

    fn refresh_diagnostics(&mut self) {
        self.diagnostics = analyze_graph(&self.document);
        if let Err(error) = export_szs(&self.document) {
            self.diagnostics.push(Diagnostic::error(
                format!("compile check failed: {error:#}"),
                None,
            ));
        }
    }

    fn push_node(&mut self, kind: EditorNodeKind) {
        let id = self.document.push_node(kind, None);
        self.selected_node = self.document.nodes.iter().position(|node| node.id == id);
        self.refresh_diagnostics();
    }
}

fn suzu_script_version() -> u32 {
    1
}
