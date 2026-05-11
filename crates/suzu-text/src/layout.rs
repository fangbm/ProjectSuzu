use serde::{Deserialize, Serialize};
use suzu_core::{Color, Rect, Vec2};

use crate::{RevealState, RubyAnnotation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WritingMode {
    Horizontal,
    VerticalRl,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextSegment {
    Plain(String),
    Ruby {
        base: String,
        ruby: String,
    },
    Color(Color),
    Size(f32),
    Bold,
    Italic,
    Variable(String),
    VoiceSync {
        char_index: usize,
        voice_file: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextBlock {
    pub raw: String,
    pub segments: Vec<TextSegment>,
    pub writing_mode: WritingMode,
    pub bounds: Rect,
    pub ruby_map: Vec<RubyAnnotation>,
    pub reveal: RevealState,
    pub wait_points: Vec<usize>,
}

impl TextBlock {
    pub fn plain(raw: impl Into<String>, bounds: Rect) -> Self {
        let markup = parse_text_markup(&raw.into());
        let raw = markup.text;
        let total_chars = raw.chars().count();
        Self {
            segments: vec![TextSegment::Plain(raw.clone())],
            raw,
            writing_mode: WritingMode::Horizontal,
            bounds,
            ruby_map: markup.ruby_map,
            reveal: RevealState::new(total_chars),
            wait_points: markup.wait_points,
        }
    }

    pub fn with_writing_mode(mut self, writing_mode: WritingMode) -> Self {
        self.writing_mode = writing_mode;
        self
    }

    pub fn visible_text(&self) -> String {
        self.raw.chars().take(self.reveal.revealed_chars).collect()
    }

    pub fn advance_reveal(&mut self, delta_ms: u32) {
        let before = self.reveal.revealed_chars;
        self.reveal.advance(delta_ms);
        if self.reveal.revealed_chars <= before {
            return;
        }

        if let Some(wait_point) = self.next_wait_point_after(before) {
            if self.reveal.revealed_chars >= wait_point {
                self.reveal.revealed_chars = wait_point;
                self.reveal.waiting_click = true;
            }
        }
    }

    pub fn reveal_to_next_wait(&mut self) {
        let next_wait = self.next_wait_point_after(self.reveal.revealed_chars);
        self.reveal.revealed_chars = next_wait.unwrap_or(self.reveal.total_chars);
        self.reveal.waiting_click = true;
    }

    pub fn continue_after_wait(&mut self) -> bool {
        if self.reveal.is_complete() || !self.reveal.waiting_click {
            return false;
        }

        self.reveal.waiting_click = false;
        true
    }

    pub fn shift_wait_points(&mut self, offset: usize) {
        for point in &mut self.wait_points {
            *point += offset;
        }
    }

    fn next_wait_point_after(&self, revealed_chars: usize) -> Option<usize> {
        self.wait_points
            .iter()
            .copied()
            .find(|point| *point > revealed_chars)
    }

    pub fn glyph_positions(&self, advance: f32) -> Vec<GlyphPosition> {
        glyph_positions(&self.raw, self.bounds, self.writing_mode, advance.max(1.0))
    }
}

pub fn normalize_text_markup(source: &str) -> String {
    parse_text_markup(source).text
}

pub fn parse_ruby_annotations(source: &str) -> Vec<RubyAnnotation> {
    parse_text_markup(source).ruby_map
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphPosition {
    pub char_index: usize,
    pub position: Vec2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedTextMarkup {
    text: String,
    wait_points: Vec<usize>,
    ruby_map: Vec<RubyAnnotation>,
}

fn parse_text_markup(source: &str) -> ParsedTextMarkup {
    let mut output = String::new();
    let mut wait_points = Vec::new();
    let mut ruby_map = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '[' {
            output.push(ch);
            continue;
        }

        let mut tag = String::new();
        let mut closed = false;
        for next in chars.by_ref() {
            if next == ']' {
                closed = true;
                break;
            }
            tag.push(next);
        }

        if !closed {
            output.push('[');
            output.push_str(&tag);
            break;
        }

        if let Some(ruby) = tag.strip_prefix("ruby=") {
            let base_start = output.chars().count();
            let mut base = String::new();
            let mut closed_ruby = false;
            while let Some(next) = chars.next() {
                if next == '[' {
                    let mut closing = String::new();
                    let mut closed_tag = false;
                    for tag_ch in chars.by_ref() {
                        if tag_ch == ']' {
                            closed_tag = true;
                            break;
                        }
                        closing.push(tag_ch);
                    }
                    if closed_tag && closing == "/ruby" {
                        closed_ruby = true;
                        break;
                    }
                    base.push('[');
                    base.push_str(&closing);
                    if closed_tag {
                        base.push(']');
                    }
                    continue;
                }
                base.push(next);
            }

            output.push_str(&base);
            if closed_ruby {
                ruby_map.push(RubyAnnotation {
                    base_range: base_start..base_start + base.chars().count(),
                    ruby: ruby.to_owned(),
                });
            } else {
                output.push_str("[/ruby]");
            }
            continue;
        }

        match tag.as_str() {
            "l" => wait_points.push(output.chars().count()),
            "r" => output.push('\n'),
            _ => {
                output.push('[');
                output.push_str(&tag);
                output.push(']');
            }
        }
    }

    ParsedTextMarkup {
        text: output,
        wait_points,
        ruby_map,
    }
}

fn glyph_positions(
    text: &str,
    bounds: Rect,
    writing_mode: WritingMode,
    advance: f32,
) -> Vec<GlyphPosition> {
    text.chars()
        .enumerate()
        .map(|(char_index, _ch)| {
            let position = match writing_mode {
                WritingMode::Horizontal => Vec2::new(
                    bounds.origin.x + char_index as f32 * advance,
                    bounds.origin.y,
                ),
                WritingMode::VerticalRl => Vec2::new(
                    bounds.origin.x + bounds.size.x - advance,
                    bounds.origin.y + char_index as f32 * advance,
                ),
            };
            GlyphPosition {
                char_index,
                position,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_strips_wait_and_converts_line_break_tags() {
        let block = TextBlock::plain("你好[l][r]下一行", Rect::default());

        assert_eq!(block.raw, "你好\n下一行");
        assert_eq!(block.reveal.total_chars, 6);
        assert_eq!(block.wait_points, vec![2]);
    }

    #[test]
    fn ruby_markup_is_stripped_and_recorded() {
        let block = TextBlock::plain("[ruby=すず]鈴[/ruby]の音", Rect::default());

        assert_eq!(block.raw, "鈴の音");
        assert_eq!(block.ruby_map.len(), 1);
        assert_eq!(block.ruby_map[0].base_range, 0..1);
        assert_eq!(block.ruby_map[0].ruby, "すず");
        assert_eq!(
            parse_ruby_annotations("[ruby=すず]鈴[/ruby]")[0].base_range,
            0..1
        );
    }

    #[test]
    fn vertical_rl_positions_glyphs_top_to_bottom_from_right_edge() {
        let block = TextBlock::plain("ABC", Rect::new(10.0, 20.0, 100.0, 200.0))
            .with_writing_mode(WritingMode::VerticalRl);

        let positions = block.glyph_positions(16.0);

        assert_eq!(positions[0].position, Vec2::new(94.0, 20.0));
        assert_eq!(positions[1].position, Vec2::new(94.0, 36.0));
        assert_eq!(positions[2].position, Vec2::new(94.0, 52.0));
    }

    #[test]
    fn unknown_or_unclosed_tags_are_preserved() {
        assert_eq!(normalize_text_markup("A[color=red]B"), "A[color=red]B");
        assert_eq!(normalize_text_markup("A[broken"), "A[broken");
    }

    #[test]
    fn reveal_stops_at_wait_points_and_can_continue() {
        let mut block = TextBlock::plain("前半[l]后半", Rect::default());
        block.reveal.speed_chars_per_second = 100.0;

        block.advance_reveal(1000);

        assert_eq!(block.visible_text(), "前半");
        assert!(block.reveal.waiting_click);
        assert!(!block.reveal.is_complete());

        assert!(block.continue_after_wait());
        block.advance_reveal(1000);

        assert_eq!(block.visible_text(), "前半后半");
        assert!(block.reveal.is_complete());
    }

    #[test]
    fn reveal_to_next_wait_reveals_current_segment_only() {
        let mut block = TextBlock::plain("A[l]B", Rect::default());

        block.reveal_to_next_wait();

        assert_eq!(block.visible_text(), "A");
        assert!(block.continue_after_wait());

        block.reveal_to_next_wait();
        assert_eq!(block.visible_text(), "AB");
        assert!(block.reveal.is_complete());
    }
}
