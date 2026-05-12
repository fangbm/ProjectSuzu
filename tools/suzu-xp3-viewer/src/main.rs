#![cfg_attr(windows, windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use eframe::egui;
use suzu_asset::{AssetType, TextureAsset, Xp3Archive, Xp3Decryptor, Xp3Entry, Xp3Options};

fn main() -> eframe::Result<()> {
    let initial_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_default()
        .display()
        .to_string();
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

struct Xp3ViewerApp {
    xp3_path: String,
    xor_enabled: bool,
    xor_key: String,
    archive: Option<Xp3Archive>,
    entries: Vec<EntryRow>,
    selected: Option<usize>,
    preview: Preview,
    status: String,
}

#[derive(Debug, Clone)]
struct EntryRow {
    name: String,
    kind: AssetType,
    encrypted: bool,
    original_size: u64,
    packed_size: u64,
}

enum Preview {
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

impl Xp3ViewerApp {
    fn new(cc: &eframe::CreationContext<'_>, initial_path: String) -> Self {
        let mut app = Self {
            xp3_path: initial_path,
            xor_enabled: false,
            xor_key: "5A".to_owned(),
            archive: None,
            entries: Vec::new(),
            selected: None,
            preview: Preview::Empty,
            status: "Enter an XP3 path and press Load.".to_owned(),
        };
        if !app.xp3_path.trim().is_empty() {
            app.load_archive(&cc.egui_ctx);
        }
        app
    }

    fn load_archive(&mut self, ctx: &egui::Context) {
        let path = PathBuf::from(self.xp3_path.trim());
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
                if !self.entries.is_empty() {
                    self.select_entry(ctx, 0);
                }
            }
            Err(error) => {
                self.archive = None;
                self.entries.clear();
                self.selected = None;
                self.preview = Preview::Empty;
                self.status = format!("Failed to load XP3: {error:#}");
            }
        }
    }

    fn xp3_options(&self) -> Result<Xp3Options, String> {
        if !self.xor_enabled {
            return Ok(Xp3Options::default());
        }

        let key_text = self.xor_key.trim().trim_start_matches("0x");
        let key = u8::from_str_radix(key_text, 16)
            .or_else(|_| self.xor_key.trim().parse::<u8>())
            .map_err(|_| "XOR key must be a byte, for example 5A or 90.".to_owned())?;
        Ok(Xp3Options {
            decryptor: Xp3Decryptor::Xor { key },
        })
    }

    fn select_entry(&mut self, ctx: &egui::Context, index: usize) {
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

    fn top_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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
            if ui.button("Load").clicked() {
                self.load_archive(ctx);
            }
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.xor_enabled, "XOR encrypted segments");
            ui.add_enabled(
                self.xor_enabled,
                egui::TextEdit::singleline(&mut self.xor_key).desired_width(60.0),
            );
            ui.separator();
            ui.label(&self.status);
        });
    }

    fn entries_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Entries");
        ui.label(format!("{} indexed", self.entries.len()));
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut clicked = None;
            for (index, row) in self.entries.iter().enumerate() {
                let selected = self.selected == Some(index);
                let marker = if row.encrypted { "locked" } else { "plain" };
                let label = format!(
                    "{:?} · {} · {} / {} bytes · {}",
                    row.kind, marker, row.packed_size, row.original_size, row.name
                );
                if ui.selectable_label(selected, label).clicked() {
                    clicked = Some(index);
                }
            }
            if let Some(index) = clicked {
                self.select_entry(ctx, index);
            }
        });
    }

    fn preview_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Preview");
        ui.separator();

        match &self.preview {
            Preview::Empty => {
                ui.label("Load an XP3 and select an entry.");
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
}

impl eframe::App for Xp3ViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| self.top_bar(ui, ctx));
        egui::SidePanel::left("entries")
            .resizable(true)
            .default_width(430.0)
            .show(ctx, |ui| self.entries_panel(ui, ctx));
        egui::CentralPanel::default().show(ctx, |ui| self.preview_panel(ui));
    }
}

impl From<&Xp3Entry> for EntryRow {
    fn from(entry: &Xp3Entry) -> Self {
        Self {
            name: entry.name.clone(),
            kind: asset_type_from_path(&entry.name),
            encrypted: entry.encrypted,
            original_size: entry.original_size,
            packed_size: entry.packed_size,
        }
    }
}

fn preview_from_bytes(ctx: &egui::Context, row: EntryRow, bytes: Vec<u8>) -> Preview {
    match row.kind {
        AssetType::Texture => match TextureAsset::from_bytes(&bytes) {
            Ok(texture) => {
                let size = [texture.width as usize, texture.height as usize];
                let image = egui::ColorImage::from_rgba_unmultiplied(size, &texture.rgba);
                let handle = ctx.load_texture(row.name.clone(), image, Default::default());
                Preview::Image {
                    name: row.name,
                    size,
                    texture: handle,
                }
            }
            Err(error) => Preview::Error {
                name: row.name,
                message: format!("{error:#}"),
            },
        },
        AssetType::Script | AssetType::Data => match String::from_utf8(bytes) {
            Ok(mut text) => {
                let truncated = text.len() > 20_000;
                if truncated {
                    text.truncate(20_000);
                }
                Preview::Text {
                    name: row.name,
                    text,
                    truncated,
                }
            }
            Err(error) => Preview::Binary {
                name: row.name,
                bytes: error.as_bytes().len(),
                kind: row.kind,
            },
        },
        kind => Preview::Binary {
            name: row.name,
            bytes: bytes.len(),
            kind,
        },
    }
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

fn fit_size(size: egui::Vec2, bounds: egui::Vec2) -> egui::Vec2 {
    let scale = (bounds.x / size.x).min(bounds.y / size.y).min(1.0);
    size * scale.max(0.01)
}
