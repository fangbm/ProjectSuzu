use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::GameState;

#[derive(Debug, Default)]
pub struct SaveManager {
    slots: Vec<Option<GameState>>,
    quicksave_slot: Option<GameState>,
    autosave_slot: Option<GameState>,
}

impl SaveManager {
    pub fn with_slots(count: usize) -> Self {
        Self {
            slots: vec![None; count],
            ..Self::default()
        }
    }

    pub fn save_slot(&mut self, index: usize, state: GameState) -> bool {
        if let Some(slot) = self.slots.get_mut(index) {
            *slot = Some(state);
            true
        } else {
            false
        }
    }

    pub fn load_slot(&self, index: usize) -> Option<&GameState> {
        self.slots.get(index).and_then(Option::as_ref)
    }

    pub fn set_quicksave(&mut self, state: GameState) {
        self.quicksave_slot = Some(state);
    }

    pub fn set_autosave(&mut self, state: GameState) {
        self.autosave_slot = Some(state);
    }

    pub fn quicksave(&self) -> Option<&GameState> {
        self.quicksave_slot.as_ref()
    }

    pub fn autosave(&self) -> Option<&GameState> {
        self.autosave_slot.as_ref()
    }

    pub fn write_slot_to_path(&self, index: usize, path: impl AsRef<Path>) -> Result<bool> {
        let Some(state) = self.load_slot(index) else {
            return Ok(false);
        };
        write_state(path, state)?;
        Ok(true)
    }

    pub fn load_slot_from_path(&mut self, index: usize, path: impl AsRef<Path>) -> Result<bool> {
        let state = read_state(path)?;
        Ok(self.save_slot(index, state))
    }
}

pub fn write_state(path: impl AsRef<Path>, state: &GameState) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create save directory {}", parent.display()))?;
    }

    let json = serde_json::to_vec_pretty(state).context("failed to serialize save state")?;
    fs::write(path, json).with_context(|| format!("failed to write save {}", path.display()))
}

pub fn read_state(path: impl AsRef<Path>) -> Result<GameState> {
    let path = path.as_ref();
    let json = fs::read(path).with_context(|| format!("failed to read save {}", path.display()))?;
    serde_json::from_slice(&json)
        .with_context(|| format!("failed to parse save {}", path.display()))
}

pub fn default_save_path(root: impl AsRef<Path>, slot: usize) -> PathBuf {
    root.as_ref().join(format!("slot_{slot:03}.json"))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use suzu_audio::AudioSource;
    use suzu_script::Command;

    use super::*;
    use crate::{
        AudioState, SaveMetadata, SaveThumbnail, SceneState, ScriptState, Value,
        SAVE_FORMAT_VERSION,
    };

    #[test]
    fn writes_and_reads_state_json() {
        let path = std::env::temp_dir().join(format!(
            "suzu-save-{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let state = GameState {
            metadata: SaveMetadata {
                format_version: SAVE_FORMAT_VERSION,
                title: "chapter one".to_owned(),
                saved_at_unix_ms: 42,
                thumbnail: SaveThumbnail::new(1, 1, vec![255, 0, 0, 255]),
            },
            script: ScriptState {
                current_file: "main.szs".to_owned(),
                line_number: 3,
                pending_commands: vec![Command::StopBgm { fadeout_ms: 100 }],
                call_stack: vec![1],
            },
            scene: SceneState::default(),
            audio: AudioState {
                bgm: Some(AudioSource::File {
                    path: "bgm.ogg".to_owned(),
                    looping: true,
                }),
                voice: None,
            },
            variables: [("seen_intro".to_owned(), Value::Bool(true))].into(),
            history: Vec::new(),
            read_dialogue_keys: vec!["main.szs:2:N:Hello".to_owned()],
        };

        write_state(&path, &state).unwrap();
        let restored = read_state(&path).unwrap();
        let _ = fs::remove_file(path);

        assert_eq!(restored, state);
    }

    #[test]
    fn save_thumbnail_validates_rgba_length() {
        assert!(SaveThumbnail::new(2, 2, vec![0; 16]).is_some());
        assert!(SaveThumbnail::new(2, 2, vec![0; 15]).is_none());
    }

    #[test]
    fn returns_false_for_empty_slot_write() {
        let manager = SaveManager::with_slots(1);

        assert!(!manager
            .write_slot_to_path(0, default_save_path(std::env::temp_dir(), 0))
            .unwrap());
    }
}
