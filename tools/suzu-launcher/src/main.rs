#![cfg_attr(windows, windows_subsystem = "windows")]

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use eframe::egui;
use suzu_app::{GameConfig, SuzuApp, TitleScreenConfig};
use suzu_asset::{AssetType, Xp3Archive, Xp3Decryptor, Xp3Entry, Xp3Options};
use suzu_editor_core::ProjectIndex;
use suzu_platform::{DesktopApp, DesktopFrame, DesktopInputEvent, FrameSprite, FrameText};

fn main() -> eframe::Result<()> {
    let initial = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_default();
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
