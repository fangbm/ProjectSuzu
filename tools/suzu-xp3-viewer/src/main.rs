#![cfg_attr(windows, windows_subsystem = "windows")]

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use eframe::egui;
use suzu_app::{GameConfig, SuzuApp, TitleScreenConfig};
use suzu_asset::{AssetType, TextureAsset, Xp3Archive, Xp3Decryptor, Xp3Entry, Xp3Options};
use suzu_platform::{DesktopApp, DesktopFrame, DesktopInputEvent, FrameSprite, FrameText};

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
    game: Option<GamePreview>,
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

struct GamePreview {
    app: SuzuApp,
    script_id: String,
    textures: HashMap<String, egui::TextureHandle>,
    last_frame: Instant,
}

impl Xp3ViewerApp {
    fn new(cc: &eframe::CreationContext<'_>, initial_path: String) -> Self {
        install_cjk_fonts(&cc.egui_ctx);
        let mut app = Self {
            xp3_path: initial_path,
            xor_enabled: false,
            xor_key: "5A".to_owned(),
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

    fn load_archive(&mut self, ctx: &egui::Context) {
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

    fn start_game(&mut self) {
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

    fn selected_script_id(&self) -> Option<String> {
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

    fn load_dropped_xp3(&mut self, ctx: &egui::Context) {
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
            let can_start = self.selected_script_id().is_some();
            if ui
                .add_enabled(can_start, egui::Button::new("Start Game"))
                .clicked()
            {
                self.start_game();
            }
            if self.game.is_some() && ui.button("Stop").clicked() {
                self.game = None;
                self.status = "Stopped game preview.".to_owned();
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
        if self.game.is_some() {
            self.game_panel(ui);
            return;
        }

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

    fn game_panel(&mut self, ui: &mut egui::Ui) {
        let Some(game) = &mut self.game else {
            return;
        };

        ui.heading(format!("Playing `{}`", game.script_id));
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
        let available = ui.available_size();
        let desired = fit_size(egui::vec2(1280.0, 720.0), available);
        let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());
        if response.clicked() {
            game.app.input(DesktopInputEvent::Confirm);
        }

        render_frame(ui.painter(), rect, &frame, &mut game.textures);
        ui.ctx().request_repaint();
    }
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

impl eframe::App for Xp3ViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.load_dropped_xp3(ctx);
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

fn asset_id_from_path(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(path)
        .to_owned()
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

fn fit_size(size: egui::Vec2, bounds: egui::Vec2) -> egui::Vec2 {
    let scale = (bounds.x / size.x).min(bounds.y / size.y).min(1.0);
    size * scale.max(0.01)
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

fn color32(color: suzu_core::Color, opacity: f32) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (color.r.clamp(0.0, 1.0) * 255.0) as u8,
        (color.g.clamp(0.0, 1.0) * 255.0) as u8,
        (color.b.clamp(0.0, 1.0) * 255.0) as u8,
        ((color.a * opacity).clamp(0.0, 1.0) * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_quoted_windows_xp3_path() {
        assert_eq!(
            clean_path_input(r#""D:\games\Suzu\data.xp3""#),
            r"D:\games\Suzu\data.xp3"
        );
    }

    #[test]
    fn cleans_file_url_xp3_path() {
        assert_eq!(
            clean_path_input("file:///D:/games/Suzu/data.xp3"),
            r"D:\games\Suzu\data.xp3"
        );
    }

    #[test]
    fn rejects_non_xp3_path() {
        assert!(xp3_path_from_input(r"D:\games\Suzu\data.zip").is_err());
    }
}
