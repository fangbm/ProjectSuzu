use std::{collections::HashMap, fs, path::Path, time::Instant};

use eframe::egui;
use suzu_app::{GameConfig, SuzuApp, TitleScreenConfig};
use suzu_platform::{DesktopFrame, FrameSprite, FrameText};

pub struct GamePreview {
    pub app: SuzuApp,
    pub label: String,
    pub textures: HashMap<String, egui::TextureHandle>,
    pub last_frame: Instant,
}

impl GamePreview {
    pub fn new(app: SuzuApp, label: String) -> Self {
        Self {
            app,
            label,
            textures: HashMap::new(),
            last_frame: Instant::now(),
        }
    }
}

pub fn preview_app(subtitle: &str) -> SuzuApp {
    SuzuApp::new(GameConfig {
        title_screen: TitleScreenConfig {
            enabled: false,
            title: "Project Suzu".to_owned(),
            subtitle: subtitle.to_owned(),
            ..TitleScreenConfig::default()
        },
        ..GameConfig::default()
    })
}

pub fn render_frame(
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

pub fn fit_size(size: egui::Vec2, bounds: egui::Vec2) -> egui::Vec2 {
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

pub fn install_cjk_fonts(ctx: &egui::Context) {
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
