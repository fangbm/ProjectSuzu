use std::{collections::HashMap, time::Instant};

use eframe::egui;
use suzu_app::{GameConfig, SuzuApp, TitleScreenConfig};
use suzu_asset::{AssetType, Xp3Archive, Xp3Entry};

use crate::fonts::install_cjk_fonts;
use crate::paths::{asset_id_from_path, asset_type_from_path, xp3_path_from_input};
use crate::preview::preview_from_bytes;

pub(crate) struct Xp3ViewerApp {
    pub(crate) xp3_path: String,
    pub(crate) xp3_plugin_path: String,
    pub(crate) xp3_plugin_authorized: bool,
    pub(crate) archive: Option<Xp3Archive>,
    pub(crate) entries: Vec<EntryRow>,
    pub(crate) selected: Option<usize>,
    pub(crate) preview: Preview,
    pub(crate) game: Option<GamePreview>,
    pub(crate) status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct EntryRow {
    pub(crate) name: String,
    pub(crate) kind: AssetType,
    pub(crate) protected: bool,
    pub(crate) original_size: u64,
    pub(crate) packed_size: u64,
}

pub(crate) enum Preview {
    Empty,
    Image {
        name: String,
        size: [usize; 2],
        texture: egui::TextureHandle,
    },
    Text {
        name: String,
        text: String,
        truncated: bool,
    },
    Binary {
        name: String,
        bytes: usize,
        kind: AssetType,
    },
    Error {
        name: String,
        message: String,
    },
}

pub(crate) struct GamePreview {
    pub(crate) app: SuzuApp,
    pub(crate) script_id: String,
    pub(crate) textures: HashMap<String, egui::TextureHandle>,
    pub(crate) last_frame: Instant,
}

impl Xp3ViewerApp {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>, initial_path: String) -> Self {
        install_cjk_fonts(&cc.egui_ctx);
        let mut app = Self {
            xp3_path: initial_path,
            xp3_plugin_path: String::new(),
            xp3_plugin_authorized: false,
            archive: None,
            entries: Vec::new(),
            selected: None,
            preview: Preview::Empty,
            game: None,
            status: "Enter an XP3 path and press Load.".to_owned(),
        };
        if !app.xp3_path.trim().is_empty() {
            app.load_archive(&cc.egui_ctx);
        }
        app
    }

    pub(crate) fn load_archive(&mut self, ctx: &egui::Context) {
        let path = match xp3_path_from_input(&self.xp3_path) {
            Ok(path) => path,
            Err(error) => {
                self.status = error;
                return;
            }
        };
        self.xp3_path = path.display().to_string();
        let options = match self.xp3_options() {
            Ok(options) => options,
            Err(error) => {
                self.status = error;
                return;
            }
        };

        match Xp3Archive::from_file_with_options(&path, options) {
            Ok(archive) => {
                self.entries = archive.entries().iter().map(EntryRow::from).collect();
                self.status = format!(
                    "Loaded {} entries from {}.",
                    self.entries.len(),
                    path.display()
                );
                self.archive = Some(archive);
                self.selected = None;
                self.preview = Preview::Empty;
                self.game = None;
                if !self.entries.is_empty() {
                    self.select_entry(ctx, 0);
                }
            }
            Err(error) => {
                self.archive = None;
                self.entries.clear();
                self.selected = None;
                self.preview = Preview::Empty;
                self.game = None;
                self.status = format!("Failed to load XP3: {error:#}");
            }
        }
    }

    pub(crate) fn select_entry(&mut self, ctx: &egui::Context, index: usize) {
        self.selected = Some(index);
        let Some(archive) = &self.archive else {
            self.preview = Preview::Empty;
            return;
        };
        let Some(row) = self.entries.get(index).cloned() else {
            self.preview = Preview::Empty;
            return;
        };

        match archive.read_file(&row.name) {
            Ok(bytes) => {
                self.status = format!("Loaded {} bytes from {}.", bytes.len(), row.name);
                self.preview = preview_from_bytes(ctx, row, bytes);
            }
            Err(error) => {
                self.preview = Preview::Error {
                    name: row.name,
                    message: format!("{error:#}"),
                };
            }
        }
    }

    pub(crate) fn start_game(&mut self) {
        let path = match xp3_path_from_input(&self.xp3_path) {
            Ok(path) => path,
            Err(error) => {
                self.status = error;
                return;
            }
        };
        let Some(script_id) = self.selected_script_id() else {
            self.status =
                "No script entry found. Select a .szs script or add one to the XP3.".to_owned();
            return;
        };
        let options = match self.xp3_options() {
            Ok(options) => options,
            Err(error) => {
                self.status = error;
                return;
            }
        };

        let mut app = SuzuApp::new(GameConfig {
            title_screen: TitleScreenConfig {
                enabled: false,
                title: "Project Suzu".to_owned(),
                subtitle: "XP3 Preview".to_owned(),
            },
            ..GameConfig::default()
        });

        match app
            .register_xp3_file_with_options(&path, options)
            .and_then(|_| app.load_script_asset(script_id.as_str()))
        {
            Ok(()) => {
                app.advance_until_waiting();
                self.status = format!("Started script `{script_id}` from {}.", path.display());
                self.game = Some(GamePreview {
                    app,
                    script_id,
                    textures: HashMap::new(),
                    last_frame: Instant::now(),
                });
            }
            Err(error) => {
                self.status = format!("Failed to start game: {error:#}");
            }
        }
    }

    pub(crate) fn selected_script_id(&self) -> Option<String> {
        if let Some(index) = self.selected {
            let row = self.entries.get(index)?;
            if row.kind == AssetType::Script {
                return Some(asset_id_from_path(&row.name));
            }
        }

        self.entries
            .iter()
            .filter(|row| row.kind == AssetType::Script)
            .find(|row| {
                let id = asset_id_from_path(&row.name).to_ascii_lowercase();
                matches!(id.as_str(), "main" | "start" | "script" | "scenario")
            })
            .or_else(|| {
                self.entries
                    .iter()
                    .find(|row| row.kind == AssetType::Script)
            })
            .map(|row| asset_id_from_path(&row.name))
    }

    pub(crate) fn load_dropped_xp3(&mut self, ctx: &egui::Context) {
        let dropped_path = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .find(|path| {
                    path.extension()
                        .and_then(|extension| extension.to_str())
                        .is_some_and(|extension| extension.eq_ignore_ascii_case("xp3"))
                })
        });

        if let Some(path) = dropped_path {
            self.xp3_path = path.display().to_string();
            self.load_archive(ctx);
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
            packed_size: entry.packed_size,
        }
    }
}
