use suzu_core::{Color, Rect};
use suzu_render::SpriteLayer;
use suzu_script::ChoiceOption;
use suzu_text::TextBlock;

#[derive(Debug)]
pub struct Scene {
    pub background: Option<SpriteLayer>,
    pub outgoing_background: Option<SpriteLayer>,
    pub characters: Vec<SpriteLayer>,
    pub dialogue: Option<TextBlock>,
    pub message_box_visible: bool,
    pub dialogue_style: DialogueBoxStyle,
    pub choice: Option<ChoiceState>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            background: None,
            outgoing_background: None,
            characters: Vec::new(),
            dialogue: None,
            message_box_visible: true,
            dialogue_style: DialogueBoxStyle::default(),
            choice: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DialogueBoxStyle {
    pub box_bounds: Rect,
    pub text_bounds: Rect,
    pub speaker_bounds: Rect,
    pub prompt_bounds: Rect,
    pub box_color: Color,
    pub speaker_color: Color,
    pub prompt_text: String,
}

impl Default for DialogueBoxStyle {
    fn default() -> Self {
        Self {
            box_bounds: Rect::new(120.0, 500.0, 1040.0, 152.0),
            text_bounds: Rect::new(144.0, 548.0, 992.0, 86.0),
            speaker_bounds: Rect::new(144.0, 484.0, 220.0, 42.0),
            prompt_bounds: Rect::new(1072.0, 612.0, 72.0, 24.0),
            box_color: Color::rgba(0.02, 0.025, 0.035, 0.9),
            speaker_color: Color::rgba(0.12, 0.16, 0.24, 0.96),
            prompt_text: "next".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceState {
    pub options: Vec<ChoiceOption>,
    pub selected_index: usize,
}

impl ChoiceState {
    pub fn new(options: Vec<ChoiceOption>) -> Self {
        Self {
            options,
            selected_index: 0,
        }
    }

    pub fn selected(&self) -> Option<&ChoiceOption> {
        self.options.get(self.selected_index)
    }

    pub fn select_next(&mut self) {
        if !self.options.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.options.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.options.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.options.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }
}
