use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use suzu_audio::AudioSource;
use suzu_render::SpriteLayer;
use suzu_script::{ChoiceOption, Command};
use suzu_text::TextBlock;

pub const SAVE_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub metadata: SaveMetadata,
    pub script: ScriptState,
    pub scene: SceneState,
    pub audio: AudioState,
    pub variables: HashMap<String, Value>,
    pub history: Vec<HistoryEntry>,
    #[serde(default)]
    pub read_dialogue_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub format_version: u32,
    pub title: String,
    pub saved_at_unix_ms: u64,
    #[serde(default)]
    pub thumbnail: Option<SaveThumbnail>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveThumbnail {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl SaveThumbnail {
    pub fn new(width: u32, height: u32, rgba: Vec<u8>) -> Option<Self> {
        let expected_len = usize::try_from(width.checked_mul(height)?.checked_mul(4)?).ok()?;
        (rgba.len() == expected_len).then_some(Self {
            width,
            height,
            rgba,
        })
    }

    pub fn byte_len(&self) -> usize {
        self.rgba.len()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptState {
    pub current_file: String,
    pub line_number: usize,
    pub pending_commands: Vec<Command>,
    pub call_stack: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SceneState {
    pub background: Option<SpriteLayer>,
    pub outgoing_background: Option<SpriteLayer>,
    pub characters: Vec<SpriteLayer>,
    pub dialogue: Option<TextBlock>,
    #[serde(default = "default_message_box_visible")]
    pub message_box_visible: bool,
    pub choice: Option<ChoiceStateSnapshot>,
}

fn default_message_box_visible() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChoiceStateSnapshot {
    pub options: Vec<ChoiceOption>,
    pub selected_index: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AudioState {
    pub bgm: Option<AudioSource>,
    pub voice: Option<AudioSource>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub speaker: Option<String>,
    pub text: String,
    #[serde(default)]
    pub voice_file: Option<String>,
}
