use std::collections::HashMap;

use eframe::egui;
use suzu_asset::{AssetType, TextureAsset};
use suzu_platform::{DesktopFrame, FrameSprite, FrameText};

use crate::app::{EntryRow, Preview};

pub(crate) fn preview_from_bytes(ctx: &egui::Context, row: EntryRow, bytes: Vec<u8>) -> Preview {
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

pub(crate) fn fit_size(size: egui::Vec2, bounds: egui::Vec2) -> egui::Vec2 {
    let scale = (bounds.x / size.x).min(bounds.y / size.y).min(1.0);
    size * scale.max(0.01)
}

pub(crate) fn render_frame(
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
