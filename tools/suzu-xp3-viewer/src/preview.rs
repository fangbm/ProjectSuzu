use std::collections::HashMap;

use eframe::egui;
use encoding_rs::SHIFT_JIS;
use suzu_asset::{AssetType, TextureAsset};
use suzu_platform::{DesktopFrame, FrameSprite, FrameText};

use crate::app::{EntryRow, Preview};

const TEXT_PREVIEW_LIMIT: usize = 20_000;
const TEXT_SCORE_SAMPLE: usize = 8_000;
const SUSPICIOUS_CJK_RUN_LIMIT: usize = 96;
const SUSPICIOUS_TEXT_NOTICE: &str = "[Project Suzu XP3 Viewer: stopped text preview at a suspicious undecoded run. The external XP3 plugin may have returned partially decoded bytes, or this entry may contain an obfuscated string literal.]";

pub(crate) enum PreviewData {
    Image {
        name: String,
        size: [usize; 2],
        rgba: Vec<u8>,
    },
    Text {
        name: String,
        text: String,
        truncated: bool,
        warning: Option<String>,
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

pub(crate) fn preview_data_from_bytes(row: EntryRow, bytes: Vec<u8>) -> PreviewData {
    match row.kind {
        AssetType::Texture => match TextureAsset::from_bytes(&bytes) {
            Ok(texture) => {
                let size = [texture.width as usize, texture.height as usize];
                PreviewData::Image {
                    name: row.name,
                    size,
                    rgba: texture.rgba,
                }
            }
            Err(error) => PreviewData::Error {
                name: row.name,
                message: format!("{error:#}"),
            },
        },
        AssetType::Script | AssetType::Data => match decode_text_preview(&bytes) {
            Some((mut text, label)) => {
                let warning = prepare_decoded_text(&mut text, label);
                let truncated = truncate_preview_text(&mut text);
                PreviewData::Text {
                    name: format!("{} · {}", row.name, label),
                    text,
                    truncated,
                    warning,
                }
            }
            None => PreviewData::Binary {
                name: row.name,
                bytes: bytes.len(),
                kind: row.kind,
            },
        },
        kind => PreviewData::Binary {
            name: row.name,
            bytes: bytes.len(),
            kind,
        },
    }
}

fn decode_text_preview(bytes: &[u8]) -> Option<(String, &'static str)> {
    if bytes.is_empty() {
        return Some((String::new(), "empty text"));
    }

    let mut candidates = Vec::new();
    if let Ok(text) = std::str::from_utf8(bytes) {
        candidates.push((strip_utf8_bom(text).to_owned(), "utf-8"));
    }

    if bytes.starts_with(&[0xff, 0xfe]) {
        if let Some(text) = decode_utf16(&bytes[2..], Utf16Endian::Little) {
            candidates.push((text, "utf-16le"));
        }
    } else if bytes.starts_with(&[0xfe, 0xff]) {
        if let Some(text) = decode_utf16(&bytes[2..], Utf16Endian::Big) {
            candidates.push((text, "utf-16be"));
        }
    } else {
        if let Some(text) = decode_utf16(bytes, Utf16Endian::Little) {
            let label = if looks_like_utf16(bytes, Utf16Endian::Little) {
                "utf-16le"
            } else {
                "utf-16le-candidate"
            };
            candidates.push((text, label));
        }
        if let Some(text) = decode_utf16(bytes, Utf16Endian::Big) {
            let label = if looks_like_utf16(bytes, Utf16Endian::Big) {
                "utf-16be"
            } else {
                "utf-16be-candidate"
            };
            candidates.push((text, label));
        }
    }

    let (shift_jis, _, had_errors) = SHIFT_JIS.decode(bytes);
    if !had_errors || text_score(&shift_jis) > 200 {
        candidates.push((shift_jis.into_owned(), "shift_jis"));
    }

    candidates
        .into_iter()
        .map(|(text, label)| {
            let base_score = text_score(&text);
            let score = base_score + encoding_score_bonus(label);
            (text, label, base_score, score)
        })
        .filter(|(_, _, base_score, _)| *base_score > 20)
        .max_by_key(|(_, _, _, score)| *score)
        .map(|(text, label, _, _)| (text, label))
}

#[derive(Clone, Copy)]
enum Utf16Endian {
    Little,
    Big,
}

fn decode_utf16(bytes: &[u8], endian: Utf16Endian) -> Option<String> {
    if bytes.len() < 2 || bytes.len() % 2 != 0 {
        return None;
    }

    let units = bytes
        .chunks_exact(2)
        .map(|chunk| match endian {
            Utf16Endian::Little => u16::from_le_bytes([chunk[0], chunk[1]]),
            Utf16Endian::Big => u16::from_be_bytes([chunk[0], chunk[1]]),
        })
        .collect::<Vec<_>>();
    let text = String::from_utf16_lossy(&units);
    Some(strip_utf16_bom(&text).to_owned())
}

fn looks_like_utf16(bytes: &[u8], endian: Utf16Endian) -> bool {
    let pairs = bytes.chunks_exact(2).take(256).collect::<Vec<_>>();
    if pairs.len() < 4 {
        return false;
    }
    let zero_count = pairs
        .iter()
        .filter(|pair| match endian {
            Utf16Endian::Little => pair[1] == 0,
            Utf16Endian::Big => pair[0] == 0,
        })
        .count();
    zero_count * 4 >= pairs.len()
}

fn encoding_score_bonus(label: &str) -> i32 {
    match label {
        "utf-16le" | "utf-16be" => 1_000,
        _ => 0,
    }
}

fn prepare_decoded_text(text: &mut String, label: &str) -> Option<String> {
    if label != "shift_jis" {
        return None;
    }

    stop_at_suspicious_cjk_run(text)
        .then(|| "Suspicious Shift_JIS text run hidden from preview.".to_owned())
}

fn stop_at_suspicious_cjk_run(text: &mut String) -> bool {
    let mut run_start = 0;
    let mut run_len = 0;

    for (index, ch) in text.char_indices() {
        if is_cjk_ideograph(ch) {
            if run_len == 0 {
                run_start = index;
            }
            run_len += 1;
            if run_len >= SUSPICIOUS_CJK_RUN_LIMIT {
                text.truncate(run_start);
                if !text.ends_with('\n') {
                    text.push_str("\n\n");
                }
                text.push_str(SUSPICIOUS_TEXT_NOTICE);
                return true;
            }
        } else {
            run_len = 0;
        }
    }

    false
}

fn is_cjk_ideograph(ch: char) -> bool {
    ('\u{3400}'..='\u{9fff}').contains(&ch)
}

fn strip_utf8_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

fn strip_utf16_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

fn text_score(text: &str) -> i32 {
    let sample = text.chars().take(TEXT_SCORE_SAMPLE);
    let mut score = 0;
    let mut count = 0;

    for ch in sample {
        count += 1;
        let code = ch as u32;
        if ch == '\u{fffd}' || ch == '\0' {
            score -= 60;
        } else if ch == '\r' || ch == '\n' || ch == '\t' || ch.is_ascii_graphic() || ch == ' ' {
            score += 3;
        } else if ('\u{3040}'..='\u{30ff}').contains(&ch)
            || ('\u{3400}'..='\u{9fff}').contains(&ch)
            || ('\u{ff00}'..='\u{ffef}').contains(&ch)
        {
            score += 4;
        } else if code < 32 {
            score -= 30;
        } else {
            score += 1;
        }
    }

    if count == 0 {
        return 0;
    }

    let lowered = text
        .chars()
        .take(TEXT_SCORE_SAMPLE)
        .collect::<String>()
        .to_ascii_lowercase();
    for token in [
        "function", "var ", "storage=", "target=", "@jump", "@call", "[jump", "[call", "//",
        "\r\n", "\n",
    ] {
        score += lowered.matches(token).count() as i32 * 20;
    }

    score
}

fn truncate_preview_text(text: &mut String) -> bool {
    if text.len() <= TEXT_PREVIEW_LIMIT {
        return false;
    }

    let mut cutoff = TEXT_PREVIEW_LIMIT;
    while !text.is_char_boundary(cutoff) {
        cutoff -= 1;
    }
    text.truncate(cutoff);
    true
}

pub(crate) fn preview_from_data(ctx: &egui::Context, data: PreviewData) -> Preview {
    match data {
        PreviewData::Image { name, size, rgba } => {
            let image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
            let texture = ctx.load_texture(name.clone(), image, Default::default());
            Preview::Image {
                name,
                size,
                texture,
            }
        }
        PreviewData::Text {
            name,
            text,
            truncated,
            warning,
        } => Preview::Text {
            name,
            text,
            truncated,
            warning,
        },
        PreviewData::Binary { name, bytes, kind } => Preview::Binary { name, bytes, kind },
        PreviewData::Error { name, message } => Preview::Error { name, message },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_preview_decodes_utf16le_without_bom() {
        let row = row("font/embfontlist.tjs", AssetType::Script);
        let source = "function main() {\r\n  var value = 1;\r\n}\r\n";
        let bytes = source
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();

        let preview = preview_data_from_bytes(row, bytes);

        let PreviewData::Text { name, text, .. } = preview else {
            panic!("expected text preview");
        };
        assert!(name.contains("utf-16le"));
        assert!(text.contains("function main()"));
    }

    #[test]
    fn script_preview_decodes_shift_jis() {
        let row = row("scenario/start.ks", AssetType::Script);
        let bytes = vec![
            0x82, 0xb1, 0x82, 0xf1, 0x82, 0xc9, 0x82, 0xbf, 0x82, 0xcd, b'\r', b'\n', b'[', b'j',
            b'u', b'm', b'p', b' ',
        ];

        let preview = preview_data_from_bytes(row, bytes);

        let PreviewData::Text {
            name,
            text,
            warning,
            ..
        } = preview
        else {
            panic!("expected text preview");
        };
        assert!(name.contains("shift_jis"));
        assert!(text.contains("\u{3053}\u{3093}\u{306b}\u{3061}\u{306f}"));
        assert!(warning.is_none());
    }

    #[test]
    fn shift_jis_preview_hides_suspicious_undecoded_run() {
        let row = row("AppConfig.tjs", AssetType::Script);
        let source = format!(
            "global.ENV_GameURL = \"{}\";\r\n",
            "\u{6e6f}".repeat(SUSPICIOUS_CJK_RUN_LIMIT + 8)
        );
        let (encoded, _, had_errors) = SHIFT_JIS.encode(&source);
        assert!(!had_errors);

        let preview = preview_data_from_bytes(row, encoded.into_owned());

        let PreviewData::Text { text, warning, .. } = preview else {
            panic!("expected text preview");
        };
        assert!(warning.is_some());
        assert!(text.contains("global.ENV_GameURL = \""));
        assert!(text.contains(SUSPICIOUS_TEXT_NOTICE));
        assert!(!text.contains(&"\u{6e6f}".repeat(SUSPICIOUS_CJK_RUN_LIMIT)));
    }

    #[test]
    fn binary_data_stays_binary_when_decoding_scores_poorly() {
        let row = row("font/font1_26.tft", AssetType::Data);
        let bytes = vec![0, 159, 12, 240, 0, 8, 3, 0, 255, 0, 1, 2, 3, 4];

        let preview = preview_data_from_bytes(row, bytes);

        assert!(matches!(preview, PreviewData::Binary { .. }));
    }

    fn row(name: &str, kind: AssetType) -> EntryRow {
        EntryRow {
            name: name.to_owned(),
            kind,
            protected: false,
            original_size: 0,
            packed_size: 0,
        }
    }
}
