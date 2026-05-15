use super::*;

impl SuzuApp {
    pub fn capture_state(&self) -> GameState {
        self.capture_state_with_thumbnail(None)
    }

    pub fn capture_state_with_thumbnail(&self, thumbnail: Option<SaveThumbnail>) -> GameState {
        GameState {
            metadata: SaveMetadata {
                format_version: SAVE_FORMAT_VERSION,
                title: self.save_title.clone(),
                saved_at_unix_ms: unix_time_ms(),
                thumbnail,
            },
            script: ScriptState {
                current_file: self.config.script_entry.clone(),
                line_number: self.script.position(),
                pending_commands: self.script.commands().to_vec(),
                call_stack: self.script.call_stack().to_vec(),
            },
            scene: SceneState {
                background: self.scene.background.clone(),
                outgoing_background: self.scene.outgoing_background.clone(),
                characters: self.scene.characters.clone(),
                dialogue: self.scene.dialogue.clone(),
                message_box_visible: self.scene.message_box_visible,
                choice: self
                    .scene
                    .choice
                    .as_ref()
                    .map(|choice| ChoiceStateSnapshot {
                        options: choice.options.clone(),
                        selected_index: choice.selected_index,
                    }),
            },
            audio: AudioState {
                bgm: self.audio.bgm.current.clone(),
                voice: self.audio.voice.current.clone(),
            },
            variables: self.variables.clone(),
            history: self.history.clone(),
            read_dialogue_keys: sorted_read_dialogue_keys(&self.read_dialogue_keys),
        }
    }

    pub fn restore_state(&mut self, state: GameState) {
        self.save_title = state.metadata.title;
        self.scene.background = state.scene.background;
        self.scene.outgoing_background = state.scene.outgoing_background;
        self.scene.characters = state.scene.characters;
        self.scene.dialogue = state.scene.dialogue;
        self.scene.message_box_visible = state.scene.message_box_visible;
        self.scene.choice = state.scene.choice.map(|choice| ChoiceState {
            options: choice.options,
            selected_index: choice.selected_index,
        });
        self.audio.bgm.current = state.audio.bgm;
        self.audio.bgm.fade_state = None;
        self.audio.bgm.volume = 1.0;
        self.audio.voice.current = state.audio.voice;
        self.audio.voice.fade_state = None;
        self.audio.voice.volume = 1.0;
        self.variables = state.variables;
        self.history = state.history;
        self.read_dialogue_keys = state.read_dialogue_keys.into_iter().collect();

        if !state.script.pending_commands.is_empty() {
            self.script = CommandQueue::new(state.script.pending_commands);
            self.script.set_position(state.script.line_number);
            self.script.set_call_stack(state.script.call_stack);
        }
        self.active_animations.clear();
        self.active_effects.clear();
        self.background_transition = None;
        self.wait_timer_ms = None;
        self.pending_voice = None;
        self.skip_mode = false;
        self.current_dialogue_key = self
            .scene
            .dialogue
            .as_ref()
            .map(|dialogue| restored_dialogue_key(&self.config.script_entry, dialogue));
        self.history_visible = false;
        self.history_scroll = 0;
        self.title_screen_visible = false;
        self.title_menu_selected = 0;
        self.system_menu_visible = false;
        self.system_menu_selected = 0;
        self.auto_advance_elapsed_ms = 0;
    }

    pub fn save_slot(&mut self, index: usize) -> bool {
        self.saves.save_slot(index, self.capture_state())
    }

    pub fn save_slot_with_thumbnail(&mut self, index: usize, thumbnail: SaveThumbnail) -> bool {
        self.saves
            .save_slot(index, self.capture_state_with_thumbnail(Some(thumbnail)))
    }

    pub fn load_slot(&mut self, index: usize) -> bool {
        let Some(state) = self.saves.load_slot(index).cloned() else {
            return false;
        };
        self.restore_state(state);
        true
    }
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}
