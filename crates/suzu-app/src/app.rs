use std::{
    collections::{HashMap, HashSet},
    time::{SystemTime, UNIX_EPOCH},
};

use suzu_asset::AssetManager;
use suzu_audio::AudioSource;
use suzu_audio::AudioSystem;
use suzu_core::{Color, Rect, Vec2};
use suzu_input::{InputEvent, InputState};
use suzu_platform::{
    DesktopApp, DesktopFrame, DesktopInputEvent, FrameBlendMode, FrameSprite, FrameText,
    FrameTexture,
};
use suzu_render::{BlendMode, Easing, Renderer, SpriteLayer, Tween};
use suzu_save::{
    AudioState, ChoiceStateSnapshot, GameState, HistoryEntry, SaveManager, SaveMetadata,
    SaveThumbnail, SceneState, ScriptState, Value, SAVE_FORMAT_VERSION,
};
use suzu_script::{
    compile_script, AnimationKind, Command, CommandQueue, Position, Transition, VisualEffect,
};
use suzu_text::{normalize_text_markup, TextBlock, TextSegment};

use crate::{scene::ChoiceState, GameConfig, Scene, UserSettings};

const SYSTEM_MENU_ACTIONS: [SystemMenuAction; 6] = [
    SystemMenuAction::Settings,
    SystemMenuAction::Save,
    SystemMenuAction::Load,
    SystemMenuAction::History,
    SystemMenuAction::ReturnTitle,
    SystemMenuAction::Quit,
];

const TITLE_MENU_ACTIONS: [TitleMenuAction; 5] = [
    TitleMenuAction::Start,
    TitleMenuAction::Continue,
    TitleMenuAction::Load,
    TitleMenuAction::Settings,
    TitleMenuAction::Quit,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleMenuAction {
    Start,
    Continue,
    Load,
    Settings,
    Quit,
}

impl TitleMenuAction {
    fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Continue => "Continue",
            Self::Load => "Load",
            Self::Settings => "Settings",
            Self::Quit => "Quit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemMenuAction {
    Settings,
    Save,
    Load,
    History,
    ReturnTitle,
    Quit,
}

impl SystemMenuAction {
    fn label(self) -> &'static str {
        match self {
            Self::Settings => "Settings",
            Self::Save => "Save",
            Self::Load => "Load",
            Self::History => "History",
            Self::ReturnTitle => "Return Title",
            Self::Quit => "Quit",
        }
    }
}

pub struct SuzuApp {
    pub config: GameConfig,
    pub scene: Scene,
    pub renderer: Renderer,
    pub assets: AssetManager,
    pub audio: AudioSystem,
    pub input: InputState,
    pub saves: SaveManager,
    pub script: CommandQueue,
    pub scene_textures: Vec<FrameTexture>,
    pub variables: HashMap<String, Value>,
    pub history: Vec<HistoryEntry>,
    pub settings: UserSettings,
    save_title: String,
    active_animations: Vec<ActiveAnimation>,
    active_effects: Vec<ActiveVisualEffect>,
    background_transition: Option<BackgroundTransition>,
    wait_timer_ms: Option<u32>,
    pending_voice: Option<PendingVoice>,
    auto_mode: bool,
    auto_advance_elapsed_ms: u32,
    skip_mode: bool,
    current_dialogue_key: Option<String>,
    read_dialogue_keys: HashSet<String>,
    history_visible: bool,
    history_scroll: usize,
    title_screen_visible: bool,
    title_menu_selected: usize,
    system_menu_visible: bool,
    system_menu_selected: usize,
    quit_requested: bool,
}

impl SuzuApp {
    pub fn new(config: GameConfig) -> Self {
        let title_screen_visible = config.title_screen.enabled;
        Self {
            config,
            scene: Scene::default(),
            renderer: Renderer::new(),
            assets: AssetManager::default(),
            audio: AudioSystem::default(),
            input: InputState::default(),
            saves: SaveManager::with_slots(99),
            script: CommandQueue::default(),
            scene_textures: Vec::new(),
            variables: HashMap::new(),
            history: Vec::new(),
            settings: UserSettings::default(),
            save_title: String::new(),
            active_animations: Vec::new(),
            active_effects: Vec::new(),
            background_transition: None,
            wait_timer_ms: None,
            pending_voice: None,
            auto_mode: false,
            auto_advance_elapsed_ms: 0,
            skip_mode: false,
            current_dialogue_key: None,
            read_dialogue_keys: HashSet::new(),
            history_visible: false,
            history_scroll: 0,
            title_screen_visible,
            title_menu_selected: 0,
            system_menu_visible: false,
            system_menu_selected: 0,
            quit_requested: false,
        }
    }

    pub fn tick(&mut self, delta_ms: u32) {
        self.process_input();
        if self.title_screen_visible {
            let _stats = self.renderer.begin_frame(3);
            return;
        }
        self.audio.advance(delta_ms);
        self.advance_animations(delta_ms);
        self.advance_effects(delta_ms);
        self.advance_background_transition(delta_ms);
        self.advance_dialogue(delta_ms);
        if self.advance_wait_timer(delta_ms) {
            self.advance_until_waiting();
        }
        self.advance_skip_mode();
        self.advance_auto_mode(delta_ms);
        let layer_count = usize::from(self.scene.outgoing_background.is_some())
            + usize::from(self.scene.background.is_some())
            + self.scene.characters.len();
        let _stats = self.renderer.begin_frame(layer_count);
    }

    pub fn load_script(&mut self, source: &str) -> Result<(), suzu_script::CompileError> {
        self.script = CommandQueue::new(compile_script(source)?);
        if self.config.title_screen.enabled {
            self.title_screen_visible = true;
            self.title_menu_selected = 0;
        }
        Ok(())
    }

    pub fn advance_script(&mut self) -> bool {
        let Some((position, command)) = self.script.next_command_with_position() else {
            return false;
        };
        let command = command.clone();
        self.apply_command(position, command);
        true
    }

    pub fn advance_until_waiting(&mut self) -> bool {
        let mut advanced = false;
        while !self.is_waiting() {
            if !self.advance_script() {
                break;
            }
            advanced = true;
        }
        advanced
    }

    pub fn handle_input_event(&mut self, event: InputEvent) {
        self.input.push(event);
    }

    pub fn register_texture(
        &mut self,
        id: impl Into<suzu_asset::AssetId>,
        path: impl Into<std::path::PathBuf>,
    ) {
        self.assets.register_texture(id, path);
    }

    pub fn register_textures_from_dir(
        &mut self,
        root: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<usize> {
        self.assets.register_textures_from_dir(root)
    }

    pub fn register_asset_manifest_file(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<usize> {
        self.assets.register_manifest_file(path)
    }

    pub fn apply_user_settings(&mut self, settings: UserSettings) {
        self.audio.master_volume = settings.audio.master_volume.clamp(0.0, 1.0);
        self.audio.bgm_volume = settings.audio.bgm_volume.clamp(0.0, 1.0);
        self.audio.voice_volume = settings.audio.voice_volume.clamp(0.0, 1.0);
        self.audio.se_volume = settings.audio.se_volume.clamp(0.0, 1.0);
        self.settings = settings;
    }

    pub fn set_auto_mode(&mut self, enabled: bool) {
        self.auto_mode = enabled;
        self.auto_advance_elapsed_ms = 0;
    }

    pub fn toggle_auto_mode(&mut self) {
        self.set_auto_mode(!self.auto_mode);
    }

    pub fn auto_mode(&self) -> bool {
        self.auto_mode
    }

    pub fn set_skip_mode(&mut self, enabled: bool) {
        self.skip_mode = enabled && self.wait_timer_ms.is_none() && self.scene.choice.is_none();
        if enabled {
            self.auto_advance_elapsed_ms = 0;
        }
    }

    pub fn toggle_skip_mode(&mut self) {
        self.set_skip_mode(!self.skip_mode);
    }

    pub fn skip_mode(&self) -> bool {
        self.skip_mode
    }

    pub fn is_current_dialogue_read(&self) -> bool {
        self.current_dialogue_key
            .as_ref()
            .is_some_and(|key| self.read_dialogue_keys.contains(key))
    }

    fn apply_command(&mut self, command_position: usize, command: Command) {
        match command {
            Command::Bg {
                file,
                time_ms,
                method,
            } => {
                self.ensure_frame_texture(&file);
                self.set_background(file, time_ms, method);
            }
            Command::Char {
                name,
                face,
                pos,
                size,
                flip_x,
                layer,
            } => {
                let texture_id = character_texture_id(&name, &face);
                self.ensure_frame_texture(&texture_id);
                let position = character_position(pos);
                upsert_character(
                    &mut self.scene.characters,
                    name,
                    texture_id,
                    position,
                    size,
                    flip_x,
                    layer,
                );
            }
            Command::HideChar { name } => {
                self.hide_character(&name);
            }
            Command::Text { speaker, content } => {
                let history_voice_file =
                    self.pending_voice.as_ref().map(|voice| voice.file.clone());
                self.current_dialogue_key = Some(dialogue_key(
                    &self.config.script_entry,
                    command_position,
                    speaker.as_deref(),
                    &content,
                ));
                let history_text = normalize_text_markup(&content);
                self.history.push(HistoryEntry {
                    speaker: speaker.clone(),
                    text: history_text,
                    voice_file: history_voice_file,
                });
                let mut block = TextBlock::plain(content, Rect::new(120.0, 500.0, 1040.0, 160.0));
                if let Some(speaker) = speaker {
                    let prefix = format!("{speaker}: ");
                    block.shift_wait_points(prefix.chars().count());
                    block.raw = format!("{prefix}{}", block.raw);
                    block.reveal.total_chars = block.raw.chars().count();
                }
                block.reveal.speed_chars_per_second =
                    self.settings.text.speed_chars_per_second.max(1.0);
                if let Some(voice) = self.pending_voice.take() {
                    block.segments.push(TextSegment::VoiceSync {
                        char_index: 0,
                        voice_file: voice.file.clone(),
                    });
                    self.play_voice(voice.file, voice.fadein_ms);
                }
                self.scene.dialogue = Some(block);
                self.scene.message_box_visible = true;
                self.auto_advance_elapsed_ms = 0;
            }
            Command::PlayBgm {
                file,
                looping,
                fadein_ms,
            } => {
                self.audio.bgm.play(
                    AudioSource::File {
                        path: file,
                        looping,
                    },
                    fadein_ms,
                );
            }
            Command::StopBgm { fadeout_ms } => {
                self.audio.bgm.stop(fadeout_ms);
            }
            Command::PlayVoice { file, fadein_ms } => {
                self.play_voice(file, fadein_ms);
            }
            Command::CueVoice { file, fadein_ms } => {
                self.pending_voice = Some(PendingVoice { file, fadein_ms });
            }
            Command::StopVoice { fadeout_ms } => {
                self.audio.voice.stop(fadeout_ms);
            }
            Command::Wait { duration_ms } => {
                self.wait_timer_ms = (duration_ms > 0).then_some(duration_ms);
            }
            Command::MessageBox { visible } => {
                self.scene.message_box_visible = visible;
            }
            Command::Anim { target, animation } => {
                self.start_animation(&target, animation.kind, animation.duration_ms)
            }
            Command::Choice { options } => {
                let options = options
                    .into_iter()
                    .filter(|option| {
                        option.condition.as_ref().map_or(true, |condition| {
                            evaluate_condition(condition, &self.variables)
                        })
                    })
                    .collect::<Vec<_>>();
                self.scene.choice = (!options.is_empty()).then(|| ChoiceState::new(options));
                self.auto_advance_elapsed_ms = 0;
                if self.scene.choice.is_some() {
                    self.skip_mode = false;
                }
            }
            Command::If {
                condition,
                then_commands,
                else_commands,
            } => {
                if evaluate_condition(&condition, &self.variables) {
                    self.script.insert_next(then_commands);
                } else {
                    self.script.insert_next(else_commands);
                }
            }
            Command::Jump { label } => {
                self.script.jump_to(&label);
            }
            Command::Call { label } => {
                self.script.call(&label);
            }
            Command::Return => {
                self.script.return_from_call();
            }
            Command::SetVar { name, value } => {
                self.variables.insert(name, parse_value(&value));
            }
            Command::Fx { effect } => self.start_visual_effect(effect),
            Command::Label { .. } => {}
            Command::SaveName { text } => {
                self.save_title = text;
            }
            Command::AutoSave => {
                let state = self.capture_state();
                self.saves.set_autosave(state);
            }
            Command::Custom { .. } => {}
        }
    }

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

    pub fn show_title_screen(&mut self) {
        self.reset_runtime_to_script_start();
        self.title_screen_visible = true;
    }

    pub fn title_screen_visible(&self) -> bool {
        self.title_screen_visible
    }

    pub fn selected_title_menu_action(&self) -> TitleMenuAction {
        TITLE_MENU_ACTIONS[self.title_menu_selected]
    }

    pub fn move_title_menu_selection(&mut self, delta: i32) {
        self.title_menu_selected =
            wrapped_index(self.title_menu_selected, TITLE_MENU_ACTIONS.len(), delta);
    }

    pub fn start_game(&mut self) {
        self.reset_runtime_to_script_start();
        self.title_screen_visible = false;
        self.advance_until_waiting();
    }

    pub fn activate_title_menu_selection(&mut self) -> TitleMenuAction {
        let action = self.selected_title_menu_action();
        self.activate_title_menu_action(action);
        action
    }

    pub fn activate_title_menu_action(&mut self, action: TitleMenuAction) {
        match action {
            TitleMenuAction::Start => self.start_game(),
            TitleMenuAction::Continue => {
                if let Some(state) = self
                    .saves
                    .autosave()
                    .cloned()
                    .or_else(|| self.saves.load_slot(0).cloned())
                {
                    self.restore_state(state);
                } else {
                    self.start_game();
                }
            }
            TitleMenuAction::Load => {
                let _ = self.load_slot(0);
            }
            TitleMenuAction::Settings => {}
            TitleMenuAction::Quit => {
                self.quit_requested = true;
                self.title_screen_visible = false;
            }
        }
    }

    fn reset_runtime_to_script_start(&mut self) {
        self.scene = Scene::default();
        self.audio = AudioSystem::default();
        self.script.set_position(0);
        self.script.set_call_stack(Vec::new());
        self.variables.clear();
        self.history.clear();
        self.save_title.clear();
        self.active_animations.clear();
        self.active_effects.clear();
        self.background_transition = None;
        self.wait_timer_ms = None;
        self.pending_voice = None;
        self.auto_mode = false;
        self.auto_advance_elapsed_ms = 0;
        self.skip_mode = false;
        self.current_dialogue_key = None;
        self.history_visible = false;
        self.history_scroll = 0;
        self.system_menu_visible = false;
        self.system_menu_selected = 0;
        self.title_menu_selected = 0;
    }

    pub fn open_history(&mut self) {
        self.history_visible = true;
        self.history_scroll = self
            .history_scroll
            .min(self.history.len().saturating_sub(1));
    }

    pub fn close_history(&mut self) {
        self.history_visible = false;
        self.history_scroll = 0;
    }

    pub fn toggle_history(&mut self) {
        if self.history_visible {
            self.close_history();
        } else {
            self.open_history();
        }
    }

    pub fn history_visible(&self) -> bool {
        self.history_visible
    }

    pub fn scroll_history(&mut self, delta: i32) {
        let max_scroll = self.history.len().saturating_sub(1);
        self.history_scroll = if delta >= 0 {
            self.history_scroll.saturating_add(delta as usize)
        } else {
            self.history_scroll
                .saturating_sub(delta.unsigned_abs() as usize)
        }
        .min(max_scroll);
    }

    pub fn visible_history_entries(&self, limit: usize) -> Vec<&HistoryEntry> {
        if limit == 0 || self.history.is_empty() {
            return Vec::new();
        }

        let end = self.history.len().saturating_sub(self.history_scroll);
        let start = end.saturating_sub(limit);
        self.history[start..end].iter().rev().collect()
    }

    pub fn replay_history_voice(&mut self, visible_index: usize) -> bool {
        let Some(voice_file) = self
            .visible_history_entries(usize::MAX)
            .get(visible_index)
            .and_then(|entry| entry.voice_file.clone())
        else {
            return false;
        };

        self.play_voice(voice_file, 0);
        true
    }

    pub fn open_system_menu(&mut self) {
        self.system_menu_visible = true;
        self.history_visible = false;
    }

    pub fn close_system_menu(&mut self) {
        self.system_menu_visible = false;
        self.system_menu_selected = 0;
    }

    pub fn system_menu_visible(&self) -> bool {
        self.system_menu_visible
    }

    pub fn quit_requested(&self) -> bool {
        self.quit_requested
    }

    pub fn selected_system_menu_action(&self) -> SystemMenuAction {
        SYSTEM_MENU_ACTIONS[self.system_menu_selected]
    }

    pub fn move_system_menu_selection(&mut self, delta: i32) {
        self.system_menu_selected =
            wrapped_index(self.system_menu_selected, SYSTEM_MENU_ACTIONS.len(), delta);
    }

    pub fn activate_system_menu_selection(&mut self) -> SystemMenuAction {
        let action = self.selected_system_menu_action();
        self.activate_system_menu_action(action);
        action
    }

    pub fn activate_system_menu_action(&mut self, action: SystemMenuAction) {
        match action {
            SystemMenuAction::Settings => {}
            SystemMenuAction::Save => {
                let _ = self.save_slot(0);
                self.close_system_menu();
            }
            SystemMenuAction::Load => {
                let _ = self.load_slot(0);
                self.close_system_menu();
            }
            SystemMenuAction::History => {
                self.close_system_menu();
                self.open_history();
            }
            SystemMenuAction::ReturnTitle => {
                self.close_system_menu();
                if self.config.title_screen.enabled {
                    self.show_title_screen();
                } else {
                    self.start_game();
                }
            }
            SystemMenuAction::Quit => {
                self.quit_requested = true;
                self.close_system_menu();
            }
        }
    }

    fn set_background(&mut self, file: String, time_ms: u32, method: Transition) {
        let mut incoming = sprite(file, Vec2::ZERO, Vec2::new(1280.0, 720.0), 0);
        let duration_ms = match method {
            Transition::CrossFade { duration_ms } => duration_ms,
            Transition::FadeThroughColor { duration_ms, .. } => duration_ms,
            Transition::Instant => time_ms,
        };

        let should_transition = duration_ms > 0 && self.scene.background.is_some();
        if should_transition {
            incoming.opacity = 0.0;
            self.scene.outgoing_background = self.scene.background.take();
            if let Some(outgoing) = self.scene.outgoing_background.as_mut() {
                outgoing.opacity = 1.0;
            }
            let kind = match method {
                Transition::CrossFade { .. } | Transition::Instant => {
                    BackgroundTransitionKind::CrossFade
                }
                Transition::FadeThroughColor { color, .. } => {
                    BackgroundTransitionKind::FadeThroughColor { color }
                }
            };
            self.background_transition = Some(BackgroundTransition {
                progress: Tween::new(0.0, 1.0, duration_ms, Easing::EaseOutQuad),
                kind,
            });
        } else {
            self.scene.outgoing_background = None;
            self.background_transition = None;
            incoming.opacity = 1.0;
        }

        self.scene.background = Some(incoming);
    }

    fn advance_background_transition(&mut self, delta_ms: u32) {
        let Some(transition) = self.background_transition.as_mut() else {
            return;
        };

        let progress = transition.progress.advance(delta_ms);
        match transition.kind {
            BackgroundTransitionKind::CrossFade => {
                if let Some(background) = self.scene.background.as_mut() {
                    background.opacity = progress;
                }
                if let Some(outgoing) = self.scene.outgoing_background.as_mut() {
                    outgoing.opacity = 1.0 - progress;
                }
            }
            BackgroundTransitionKind::FadeThroughColor { .. } => {
                let incoming_opacity = ((progress - 0.5) * 2.0).clamp(0.0, 1.0);
                let outgoing_opacity = (1.0 - progress * 2.0).clamp(0.0, 1.0);
                if let Some(background) = self.scene.background.as_mut() {
                    background.opacity = incoming_opacity;
                }
                if let Some(outgoing) = self.scene.outgoing_background.as_mut() {
                    outgoing.opacity = outgoing_opacity;
                }
            }
        }

        if transition.progress.is_finished() {
            if let Some(background) = self.scene.background.as_mut() {
                background.opacity = 1.0;
            }
            self.scene.outgoing_background = None;
            self.background_transition = None;
        }
    }

    fn advance_dialogue(&mut self, delta_ms: u32) {
        if let Some(dialogue) = self.scene.dialogue.as_mut() {
            dialogue.advance_reveal(delta_ms);
        }
        self.mark_current_dialogue_read_if_complete();
    }

    fn advance_skip_mode(&mut self) {
        if !self.skip_mode || self.wait_timer_ms.is_some() || self.scene.choice.is_some() {
            return;
        }

        for _ in 0..32 {
            if !self.can_skip_current_dialogue() {
                break;
            }

            if self
                .scene
                .dialogue
                .as_ref()
                .is_some_and(|dialogue| !dialogue.reveal.is_complete())
            {
                self.reveal_dialogue_now();
            }

            if !self.can_skip_current_dialogue() {
                break;
            }

            self.confirm();

            if self.wait_timer_ms.is_some() || self.scene.choice.is_some() {
                break;
            }
        }
    }

    fn can_skip_current_dialogue(&self) -> bool {
        if self.wait_timer_ms.is_some() || self.scene.choice.is_some() {
            return false;
        }

        self.scene.dialogue.is_some() && self.is_current_dialogue_read()
    }

    fn advance_auto_mode(&mut self, delta_ms: u32) {
        if !self.auto_mode || !self.can_auto_advance_dialogue() {
            self.auto_advance_elapsed_ms = 0;
            return;
        }

        self.auto_advance_elapsed_ms = self.auto_advance_elapsed_ms.saturating_add(delta_ms);
        let delay_ms = self.settings.text.auto_advance_delay_ms;
        if self.auto_advance_elapsed_ms >= delay_ms {
            self.auto_advance_elapsed_ms = 0;
            self.confirm();
        }
    }

    fn can_auto_advance_dialogue(&self) -> bool {
        if self.wait_timer_ms.is_some() || self.scene.choice.is_some() {
            return false;
        }

        self.scene
            .dialogue
            .as_ref()
            .is_some_and(|dialogue| dialogue.reveal.is_complete())
    }

    fn play_voice(&mut self, file: String, fadein_ms: u32) {
        self.audio.voice.play(
            AudioSource::File {
                path: file,
                looping: false,
            },
            fadein_ms,
        );
    }

    pub fn reveal_dialogue_now(&mut self) {
        if let Some(dialogue) = self.scene.dialogue.as_mut() {
            dialogue.reveal_to_next_wait();
        }
        self.mark_current_dialogue_read_if_complete();
    }

    fn process_input(&mut self) {
        let events = self.input.drain().collect::<Vec<_>>();
        for event in events {
            if self.title_screen_visible {
                match event {
                    InputEvent::Cancel => self.activate_title_menu_action(TitleMenuAction::Quit),
                    InputEvent::Confirm
                    | InputEvent::PointerDown { .. }
                    | InputEvent::TouchStart { .. } => {
                        self.activate_title_menu_selection();
                    }
                    InputEvent::Scroll { delta } => {
                        self.move_title_menu_selection(if delta < 0.0 { 1 } else { -1 });
                    }
                    InputEvent::MoveSelection { delta } => self.move_title_menu_selection(delta),
                    InputEvent::PointerUp { .. }
                    | InputEvent::TouchMove { .. }
                    | InputEvent::TouchEnd { .. } => {}
                }
                continue;
            }

            if self.system_menu_visible {
                match event {
                    InputEvent::Cancel => self.close_system_menu(),
                    InputEvent::Confirm
                    | InputEvent::PointerDown { .. }
                    | InputEvent::TouchStart { .. } => {
                        self.activate_system_menu_selection();
                    }
                    InputEvent::Scroll { delta } => {
                        self.move_system_menu_selection(if delta < 0.0 { 1 } else { -1 });
                    }
                    InputEvent::MoveSelection { delta } => self.move_system_menu_selection(delta),
                    InputEvent::PointerUp { .. }
                    | InputEvent::TouchMove { .. }
                    | InputEvent::TouchEnd { .. } => {}
                }
                continue;
            }

            if self.history_visible {
                match event {
                    InputEvent::Cancel => self.close_history(),
                    InputEvent::Scroll { delta } => {
                        self.scroll_history(if delta < 0.0 { 1 } else { -1 });
                    }
                    InputEvent::MoveSelection { delta } => self.scroll_history(delta),
                    InputEvent::Confirm
                    | InputEvent::PointerDown { .. }
                    | InputEvent::TouchStart { .. } => {
                        let _ = self.replay_history_voice(0);
                    }
                    InputEvent::PointerUp { .. }
                    | InputEvent::TouchMove { .. }
                    | InputEvent::TouchEnd { .. } => {}
                }
                continue;
            }

            match event {
                InputEvent::Confirm
                | InputEvent::PointerDown { .. }
                | InputEvent::TouchStart { .. } => self.confirm(),
                InputEvent::Scroll { delta } => self.scroll_choice(delta),
                InputEvent::MoveSelection { delta } => self.move_choice(delta),
                InputEvent::Cancel => self.open_system_menu(),
                InputEvent::PointerUp { .. }
                | InputEvent::TouchMove { .. }
                | InputEvent::TouchEnd { .. } => {}
            }
        }
    }

    pub fn confirm(&mut self) {
        self.auto_advance_elapsed_ms = 0;
        if self.wait_timer_ms.is_some() {
            return;
        }

        if self.confirm_choice() {
            return;
        }

        if self
            .scene
            .dialogue
            .as_ref()
            .is_some_and(|dialogue| !dialogue.reveal.is_complete())
        {
            let dialogue = self
                .scene
                .dialogue
                .as_mut()
                .expect("dialogue exists after is_some_and");
            if !dialogue.continue_after_wait() {
                dialogue.reveal_to_next_wait();
            }
            self.mark_current_dialogue_read_if_complete();
        } else {
            self.advance_until_waiting();
        }
    }

    fn confirm_choice(&mut self) -> bool {
        let Some(choice) = self.scene.choice.take() else {
            return false;
        };
        self.skip_mode = false;
        let Some(selected) = choice.selected() else {
            return false;
        };
        self.script.jump_to(&selected.goto);
        self.advance_until_waiting();
        true
    }

    fn mark_current_dialogue_read_if_complete(&mut self) {
        if self
            .scene
            .dialogue
            .as_ref()
            .is_some_and(|dialogue| dialogue.reveal.is_complete())
        {
            if let Some(key) = &self.current_dialogue_key {
                self.read_dialogue_keys.insert(key.clone());
            }
        }
    }

    fn scroll_choice(&mut self, delta: f32) {
        self.move_choice(if delta < 0.0 {
            1
        } else if delta > 0.0 {
            -1
        } else {
            0
        });
    }

    fn move_choice(&mut self, delta: i32) {
        let Some(choice) = self.scene.choice.as_mut() else {
            return;
        };
        match delta.cmp(&0) {
            std::cmp::Ordering::Greater => {
                for _ in 0..delta {
                    choice.select_next();
                }
            }
            std::cmp::Ordering::Less => {
                for _ in 0..delta.unsigned_abs() {
                    choice.select_previous();
                }
            }
            std::cmp::Ordering::Equal => {}
        }
    }

    fn is_waiting_for_dialogue(&self) -> bool {
        self.scene
            .dialogue
            .as_ref()
            .is_some_and(|dialogue| !dialogue.reveal.is_complete())
    }

    fn is_waiting_for_choice(&self) -> bool {
        self.scene.choice.is_some()
    }

    fn is_waiting(&self) -> bool {
        self.wait_timer_ms.is_some()
            || self.is_waiting_for_dialogue()
            || self.is_waiting_for_choice()
    }

    fn advance_wait_timer(&mut self, delta_ms: u32) -> bool {
        let Some(remaining_ms) = self.wait_timer_ms.as_mut() else {
            return false;
        };

        *remaining_ms = remaining_ms.saturating_sub(delta_ms);
        if *remaining_ms > 0 {
            return false;
        }

        self.wait_timer_ms = None;
        true
    }

    fn start_animation(&mut self, target: &str, kind: AnimationKind, duration_ms: u32) {
        let Some(snapshot) = self.layer_snapshot(target) else {
            return;
        };

        if duration_ms == 0 {
            self.apply_animation_immediately(target, kind);
            return;
        }

        let animation = match kind {
            AnimationKind::MoveTo { position } => ActiveAnimationKind::MoveTo {
                x: Tween::new(
                    snapshot.position.x,
                    position.x,
                    duration_ms,
                    Easing::EaseOutQuad,
                ),
                y: Tween::new(
                    snapshot.position.y,
                    position.y,
                    duration_ms,
                    Easing::EaseOutQuad,
                ),
            },
            AnimationKind::Zoom { scale, .. } => ActiveAnimationKind::Scale {
                x: Tween::new(snapshot.scale.x, scale, duration_ms, Easing::EaseOutQuad),
                y: Tween::new(snapshot.scale.y, scale, duration_ms, Easing::EaseOutQuad),
            },
            AnimationKind::Shake { intensity } => ActiveAnimationKind::Rotation {
                radians: Tween::new(
                    snapshot.rotation,
                    intensity.to_radians(),
                    duration_ms,
                    Easing::EaseOutQuad,
                ),
            },
            AnimationKind::FadeTo { opacity } => ActiveAnimationKind::Opacity {
                value: Tween::new(
                    snapshot.opacity,
                    opacity.clamp(0.0, 1.0),
                    duration_ms,
                    Easing::EaseOutQuad,
                ),
            },
        };

        self.active_animations.push(ActiveAnimation {
            target: target.to_owned(),
            kind: animation,
        });
    }

    fn hide_character(&mut self, name: &str) {
        let Some(index) = self
            .scene
            .characters
            .iter()
            .position(|character| character.entity_id.as_deref() == Some(name))
        else {
            return;
        };

        let removed = self.scene.characters.remove(index);
        self.active_animations
            .retain(|animation| !layer_matches_target(&removed, &animation.target));
    }

    fn apply_animation_immediately(&mut self, target: &str, kind: AnimationKind) {
        let Some(layer) = self.layer_mut(target) else {
            return;
        };

        match kind {
            AnimationKind::MoveTo { position } => layer.position = position,
            AnimationKind::Zoom { scale, .. } => layer.scale = Vec2::new(scale, scale),
            AnimationKind::Shake { intensity } => layer.rotation = intensity.to_radians(),
            AnimationKind::FadeTo { opacity } => layer.opacity = opacity.clamp(0.0, 1.0),
        }
    }

    fn advance_animations(&mut self, delta_ms: u32) {
        let mut animations = std::mem::take(&mut self.active_animations);
        for animation in &mut animations {
            let Some(layer) = self.layer_mut(&animation.target) else {
                animation.kind.finish();
                continue;
            };
            animation.kind.apply(layer, delta_ms);
        }
        animations.retain(|animation| !animation.kind.is_finished());
        self.active_animations = animations;
    }

    fn start_visual_effect(&mut self, effect: VisualEffect) {
        let effect = match effect {
            VisualEffect::Flash { color, duration_ms } => ActiveVisualEffect::Flash {
                color,
                opacity: Tween::new(1.0, 0.0, duration_ms.max(1), Easing::Linear),
            },
            VisualEffect::Quake {
                intensity,
                duration_ms,
            } => ActiveVisualEffect::Quake {
                intensity,
                duration_ms: duration_ms.max(1),
                elapsed_ms: 0,
            },
        };
        self.active_effects.push(effect);
    }

    fn advance_effects(&mut self, delta_ms: u32) {
        for effect in &mut self.active_effects {
            effect.advance(delta_ms);
        }
        self.active_effects.retain(|effect| !effect.is_finished());
    }

    fn quake_offset(&self) -> Vec2 {
        self.active_effects
            .iter()
            .map(ActiveVisualEffect::quake_offset)
            .fold(Vec2::ZERO, |acc, offset| {
                Vec2::new(acc.x + offset.x, acc.y + offset.y)
            })
    }

    fn flash_sprite(&self) -> Option<FrameSprite> {
        self.active_effects.iter().find_map(|effect| match effect {
            ActiveVisualEffect::Flash { color, opacity } => Some(
                FrameSprite::solid(
                    "fx_flash",
                    Rect::new(0.0, 0.0, 1280.0, 720.0),
                    *color,
                    10_000,
                )
                .with_opacity(opacity.value()),
            ),
            ActiveVisualEffect::Quake { .. } => None,
        })
    }

    fn background_transition_overlay_sprite(&self) -> Option<FrameSprite> {
        let transition = self.background_transition.as_ref()?;
        let BackgroundTransitionKind::FadeThroughColor { color } = transition.kind else {
            return None;
        };

        let progress = transition.progress.value();
        let opacity = if progress <= 0.5 {
            progress * 2.0
        } else {
            (1.0 - progress) * 2.0
        }
        .clamp(0.0, 1.0);

        (opacity > 0.0).then(|| {
            FrameSprite::solid(
                "bg_transition_color",
                Rect::new(0.0, 0.0, 1280.0, 720.0),
                color,
                9_000,
            )
            .with_opacity(opacity)
        })
    }

    fn layer_snapshot(&self, target: &str) -> Option<LayerSnapshot> {
        let layer = if matches!(target, "bg" | "background") {
            self.scene.background.as_ref()
        } else {
            self.scene
                .characters
                .iter()
                .find(|character| layer_matches_target(character, target))
        }?;

        Some(LayerSnapshot {
            position: layer.position,
            scale: layer.scale,
            rotation: layer.rotation,
            opacity: layer.opacity,
        })
    }

    fn layer_mut(&mut self, target: &str) -> Option<&mut SpriteLayer> {
        if matches!(target, "bg" | "background") {
            self.scene.background.as_mut()
        } else {
            self.scene
                .characters
                .iter_mut()
                .find(|character| layer_matches_target(character, target))
        }
    }

    fn ensure_frame_texture(&mut self, id: &str) {
        if self
            .scene_textures
            .iter()
            .any(|texture| texture.id.as_str() == id)
        {
            return;
        }

        let Ok(texture) = self.assets.load_texture(id) else {
            return;
        };
        self.scene_textures.push(FrameTexture::new(
            id,
            texture.width,
            texture.height,
            texture.rgba,
        ));
    }
}

impl DesktopApp for SuzuApp {
    fn input(&mut self, event: DesktopInputEvent) {
        match event {
            DesktopInputEvent::Confirm => self.handle_input_event(InputEvent::Confirm),
            DesktopInputEvent::Cancel => self.handle_input_event(InputEvent::Cancel),
            DesktopInputEvent::MoveSelection { delta } => {
                self.handle_input_event(InputEvent::MoveSelection { delta })
            }
            DesktopInputEvent::Scroll { delta } => {
                self.handle_input_event(InputEvent::Scroll { delta })
            }
        }
    }

    fn update(&mut self, delta_ms: u32) -> DesktopFrame {
        self.tick(delta_ms);
        if self.title_screen_visible {
            return title_frame(
                &self.config.title_screen.title,
                &self.config.title_screen.subtitle,
                self.title_menu_selected,
                &self.scene_textures,
            );
        }

        let quake_offset = self.quake_offset();
        let mut sprites = Vec::new();
        if let Some(background) = &self.scene.outgoing_background {
            sprites.push(offset_sprite(
                frame_sprite(background, Color::rgba(0.22, 0.28, 0.38, 1.0)),
                quake_offset,
            ));
        }
        if let Some(background) = &self.scene.background {
            sprites.push(offset_sprite(
                frame_sprite(background, Color::rgba(0.22, 0.28, 0.38, 1.0)),
                quake_offset,
            ));
        }
        sprites.extend(self.scene.characters.iter().map(|character| {
            offset_sprite(
                frame_sprite(character, Color::rgba(0.86, 0.68, 0.74, 1.0)),
                quake_offset,
            )
        }));
        if let Some(dialogue) = self
            .scene
            .message_box_visible
            .then_some(self.scene.dialogue.as_ref())
            .flatten()
        {
            let style = &self.scene.dialogue_style;
            sprites.push(offset_sprite(
                FrameSprite::solid("dialogue", style.box_bounds, style.box_color, 100),
                quake_offset,
            ));
            let visible = dialogue.visible_text();
            let (speaker, _) = split_speaker_line(&visible);
            if speaker.is_some() {
                sprites.push(offset_sprite(
                    FrameSprite::solid(
                        "dialogue_speaker",
                        style.speaker_bounds,
                        style.speaker_color,
                        101,
                    ),
                    quake_offset,
                ));
            }
        }
        if let Some(choice) = &self.scene.choice {
            for (index, _option) in choice.options.iter().enumerate() {
                let tint = if index == choice.selected_index {
                    Color::rgba(0.18, 0.22, 0.32, 0.95)
                } else {
                    Color::rgba(0.05, 0.06, 0.08, 0.82)
                };
                sprites.push(offset_sprite(
                    FrameSprite::solid(
                        format!("choice_{index}"),
                        Rect::new(760.0, 310.0 + index as f32 * 58.0, 360.0, 44.0),
                        tint,
                        110 + index as i32,
                    ),
                    quake_offset,
                ));
            }
        }
        if self.history_visible {
            sprites.push(FrameSprite::solid(
                "history_backlog",
                Rect::new(160.0, 72.0, 960.0, 560.0),
                Color::rgba(0.015, 0.018, 0.026, 0.94),
                20_000,
            ));
        }
        if self.system_menu_visible {
            sprites.push(FrameSprite::solid(
                "system_menu",
                Rect::new(440.0, 116.0, 400.0, 488.0),
                Color::rgba(0.018, 0.022, 0.032, 0.96),
                21_000,
            ));
            sprites.push(FrameSprite::solid(
                "system_menu_selection",
                Rect::new(
                    472.0,
                    184.0 + self.system_menu_selected as f32 * 58.0,
                    336.0,
                    42.0,
                ),
                Color::rgba(0.16, 0.22, 0.32, 0.95),
                21_001,
            ));
        }
        if let Some(transition_overlay) = self.background_transition_overlay_sprite() {
            sprites.push(transition_overlay);
        }
        if let Some(flash_sprite) = self.flash_sprite() {
            sprites.push(flash_sprite);
        }
        let dialogue = self
            .scene
            .message_box_visible
            .then_some(self.scene.dialogue.as_ref())
            .flatten();
        let texts = offset_texts(
            frame_texts(
                dialogue,
                self.scene.choice.as_ref(),
                self.history_visible
                    .then(|| self.visible_history_entries(8)),
                self.system_menu_visible
                    .then_some((self.system_menu_selected, SYSTEM_MENU_ACTIONS.as_slice())),
                &self.scene.dialogue_style,
            ),
            quake_offset,
        );

        DesktopFrame {
            clear_color: Color::rgba(0.08, 0.09, 0.12, 1.0),
            textures: self.scene_textures.clone(),
            sprites,
            texts,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LayerSnapshot {
    position: Vec2,
    scale: Vec2,
    rotation: f32,
    opacity: f32,
}

#[derive(Debug, Clone)]
struct ActiveAnimation {
    target: String,
    kind: ActiveAnimationKind,
}

#[derive(Debug, Clone)]
struct PendingVoice {
    file: String,
    fadein_ms: u32,
}

#[derive(Debug, Clone)]
struct BackgroundTransition {
    progress: Tween,
    kind: BackgroundTransitionKind,
}

#[derive(Debug, Clone, Copy)]
enum BackgroundTransitionKind {
    CrossFade,
    FadeThroughColor { color: Color },
}

#[derive(Debug, Clone)]
enum ActiveVisualEffect {
    Flash {
        color: Color,
        opacity: Tween,
    },
    Quake {
        intensity: f32,
        duration_ms: u32,
        elapsed_ms: u32,
    },
}

impl ActiveVisualEffect {
    fn advance(&mut self, delta_ms: u32) {
        match self {
            Self::Flash { opacity, .. } => {
                opacity.advance(delta_ms);
            }
            Self::Quake {
                duration_ms,
                elapsed_ms,
                ..
            } => {
                *elapsed_ms = elapsed_ms.saturating_add(delta_ms).min(*duration_ms);
            }
        }
    }

    fn is_finished(&self) -> bool {
        match self {
            Self::Flash { opacity, .. } => opacity.is_finished(),
            Self::Quake {
                duration_ms,
                elapsed_ms,
                ..
            } => elapsed_ms >= duration_ms,
        }
    }

    fn quake_offset(&self) -> Vec2 {
        let Self::Quake {
            intensity,
            duration_ms,
            elapsed_ms,
        } = self
        else {
            return Vec2::ZERO;
        };

        let progress = *elapsed_ms as f32 / *duration_ms as f32;
        let falloff = 1.0 - progress.clamp(0.0, 1.0);
        let phase = *elapsed_ms as f32 / 32.0;
        Vec2::new(
            phase.sin() * *intensity * falloff,
            (phase * 1.7).cos() * *intensity * 0.5 * falloff,
        )
    }
}

#[derive(Debug, Clone)]
enum ActiveAnimationKind {
    MoveTo { x: Tween, y: Tween },
    Scale { x: Tween, y: Tween },
    Rotation { radians: Tween },
    Opacity { value: Tween },
}

impl ActiveAnimationKind {
    fn apply(&mut self, layer: &mut SpriteLayer, delta_ms: u32) {
        match self {
            Self::MoveTo { x, y } => {
                layer.position = Vec2::new(x.advance(delta_ms), y.advance(delta_ms));
            }
            Self::Scale { x, y } => {
                layer.scale = Vec2::new(x.advance(delta_ms), y.advance(delta_ms));
            }
            Self::Rotation { radians } => {
                layer.rotation = radians.advance(delta_ms);
            }
            Self::Opacity { value } => {
                layer.opacity = value.advance(delta_ms);
            }
        }
    }

    fn finish(&mut self) {
        match self {
            Self::MoveTo { x, y } | Self::Scale { x, y } => {
                x.elapsed_ms = x.duration_ms;
                y.elapsed_ms = y.duration_ms;
            }
            Self::Rotation { radians } => {
                radians.elapsed_ms = radians.duration_ms;
            }
            Self::Opacity { value } => {
                value.elapsed_ms = value.duration_ms;
            }
        }
    }

    fn is_finished(&self) -> bool {
        match self {
            Self::MoveTo { x, y } | Self::Scale { x, y } => x.is_finished() && y.is_finished(),
            Self::Rotation { radians } => radians.is_finished(),
            Self::Opacity { value } => value.is_finished(),
        }
    }
}

fn sprite(texture_id: String, position: Vec2, size: Vec2, z_index: i32) -> SpriteLayer {
    SpriteLayer {
        entity_id: None,
        texture_id,
        position,
        size,
        scale: Vec2::ONE,
        rotation: 0.0,
        opacity: 1.0,
        flip_x: false,
        blend_mode: BlendMode::Normal,
        z_index,
    }
}

fn character_sprite(
    name: String,
    texture_id: String,
    position: Vec2,
    size: Vec2,
    flip_x: bool,
    z_index: i32,
) -> SpriteLayer {
    SpriteLayer {
        entity_id: Some(name),
        texture_id,
        position,
        size,
        scale: Vec2::ONE,
        rotation: 0.0,
        opacity: 1.0,
        flip_x,
        blend_mode: BlendMode::Normal,
        z_index,
    }
}

fn upsert_character(
    characters: &mut Vec<SpriteLayer>,
    name: String,
    texture_id: String,
    position: Vec2,
    size: Vec2,
    flip_x: bool,
    z_index: i32,
) {
    if let Some(character) = characters
        .iter_mut()
        .find(|character| character.entity_id.as_deref() == Some(name.as_str()))
    {
        character.texture_id = texture_id;
        character.position = position;
        character.size = size;
        character.flip_x = flip_x;
        character.z_index = z_index;
        return;
    }

    characters.push(character_sprite(
        name, texture_id, position, size, flip_x, z_index,
    ));
}

fn character_texture_id(name: &str, face: &str) -> String {
    if face.is_empty() || face == "neutral" {
        name.to_owned()
    } else {
        format!("{name}_{face}")
    }
}

fn layer_matches_target(layer: &SpriteLayer, target: &str) -> bool {
    layer.texture_id == target || layer.entity_id.as_deref() == Some(target)
}

fn frame_sprite(layer: &SpriteLayer, tint: Color) -> FrameSprite {
    FrameSprite::solid(
        layer.texture_id.clone(),
        Rect {
            origin: layer.position,
            size: layer.size,
        },
        tint,
        layer.z_index,
    )
    .with_opacity(layer.opacity)
    .with_scale(layer.scale)
    .with_rotation(layer.rotation)
    .with_flip_x(layer.flip_x)
    .with_blend_mode(frame_blend_mode(layer.blend_mode))
}

fn offset_sprite(mut sprite: FrameSprite, offset: Vec2) -> FrameSprite {
    sprite.bounds.origin.x += offset.x;
    sprite.bounds.origin.y += offset.y;
    sprite
}

fn offset_texts(texts: Vec<FrameText>, offset: Vec2) -> Vec<FrameText> {
    texts
        .into_iter()
        .map(|mut text| {
            text.bounds.origin.x += offset.x;
            text.bounds.origin.y += offset.y;
            text
        })
        .collect()
}

fn frame_blend_mode(blend_mode: BlendMode) -> FrameBlendMode {
    match blend_mode {
        BlendMode::Normal => FrameBlendMode::Normal,
        BlendMode::Add => FrameBlendMode::Add,
        BlendMode::Multiply => FrameBlendMode::Multiply,
        BlendMode::Screen => FrameBlendMode::Screen,
    }
}

fn character_position(position: Position) -> Vec2 {
    match position {
        Position::Left => Vec2::new(180.0, 0.0),
        Position::Center => Vec2::new(460.0, 0.0),
        Position::Right => Vec2::new(740.0, 0.0),
        Position::Custom(value) => value,
    }
}

fn title_frame(
    title: &str,
    subtitle: &str,
    selected: usize,
    textures: &[FrameTexture],
) -> DesktopFrame {
    let mut sprites = vec![
        FrameSprite::solid(
            "title_background",
            Rect::new(0.0, 0.0, 1280.0, 720.0),
            Color::rgba(0.035, 0.04, 0.055, 1.0),
            0,
        ),
        FrameSprite::solid(
            "title_panel",
            Rect::new(720.0, 126.0, 368.0, 424.0),
            Color::rgba(0.02, 0.024, 0.034, 0.92),
            10,
        ),
        FrameSprite::solid(
            "title_accent",
            Rect::new(96.0, 560.0, 472.0, 4.0),
            Color::rgba(0.74, 0.34, 0.28, 1.0),
            11,
        ),
        FrameSprite::solid(
            "title_menu_selection",
            Rect::new(752.0, 252.0 + selected as f32 * 58.0, 304.0, 42.0),
            Color::rgba(0.18, 0.24, 0.34, 0.95),
            12,
        ),
    ];

    sprites.push(FrameSprite::solid(
        "title_glow",
        Rect::new(72.0, 80.0, 560.0, 400.0),
        Color::rgba(0.11, 0.13, 0.18, 0.72),
        1,
    ));

    let mut texts = vec![
        FrameText::new(
            title.to_owned(),
            Rect::new(96.0, 164.0, 560.0, 80.0),
            Color::rgba(0.96, 0.9, 0.78, 1.0),
            100,
        ),
        FrameText::new(
            subtitle.to_owned(),
            Rect::new(100.0, 252.0, 500.0, 36.0),
            Color::rgba(0.76, 0.8, 0.88, 1.0),
            101,
        ),
        FrameText::new(
            "Title".to_owned(),
            Rect::new(752.0, 158.0, 304.0, 42.0),
            Color::rgba(0.88, 0.9, 0.96, 1.0),
            102,
        ),
    ];

    for (index, action) in TITLE_MENU_ACTIONS.iter().enumerate() {
        let marker = if index == selected { "> " } else { "  " };
        let color = if index == selected {
            Color::WHITE
        } else {
            Color::rgba(0.76, 0.8, 0.88, 1.0)
        };
        texts.push(FrameText::new(
            format!("{marker}{}", action.label()),
            Rect::new(776.0, 260.0 + index as f32 * 58.0, 256.0, 30.0),
            color,
            110 + index as i32,
        ));
    }

    DesktopFrame {
        clear_color: Color::rgba(0.035, 0.04, 0.055, 1.0),
        textures: textures.to_vec(),
        sprites,
        texts,
    }
}

fn frame_texts(
    dialogue: Option<&TextBlock>,
    choice: Option<&ChoiceState>,
    history_entries: Option<Vec<&HistoryEntry>>,
    system_menu: Option<(usize, &[SystemMenuAction])>,
    dialogue_style: &crate::scene::DialogueBoxStyle,
) -> Vec<FrameText> {
    let mut texts = Vec::new();
    if let Some(dialogue) = dialogue {
        let visible = dialogue.visible_text();
        let (speaker, content) = split_speaker_line(&visible);
        if let Some(speaker) = speaker {
            texts.push(FrameText::new(
                speaker.to_owned(),
                dialogue_style.speaker_bounds,
                Color::WHITE,
                121,
            ));
        }
        texts.push(FrameText::new(
            content.to_owned(),
            dialogue_style.text_bounds,
            Color::WHITE,
            120,
        ));
        if dialogue.reveal.is_complete() {
            texts.push(FrameText::new(
                dialogue_style.prompt_text.clone(),
                dialogue_style.prompt_bounds,
                Color::rgba(0.78, 0.84, 0.94, 1.0),
                122,
            ));
        }
    }
    if let Some(choice) = choice {
        for (index, option) in choice.options.iter().enumerate() {
            let color = if index == choice.selected_index {
                Color::WHITE
            } else {
                Color::rgba(0.72, 0.76, 0.84, 1.0)
            };
            texts.push(FrameText::new(
                option.text.clone(),
                Rect::new(784.0, 320.0 + index as f32 * 58.0, 320.0, 28.0),
                color,
                130 + index as i32,
            ));
        }
    }
    if let Some(history_entries) = history_entries {
        for (index, entry) in history_entries.iter().enumerate() {
            let speaker = entry
                .speaker
                .as_ref()
                .map(|speaker| format!("{speaker}: "))
                .unwrap_or_default();
            let voice_hint = entry
                .voice_file
                .as_ref()
                .map(|_| " [voice]")
                .unwrap_or_default();
            texts.push(FrameText::new(
                format!("{speaker}{}{voice_hint}", entry.text),
                Rect::new(192.0, 104.0 + index as f32 * 58.0, 896.0, 42.0),
                Color::rgba(0.9, 0.92, 0.96, 1.0),
                20_010 + index as i32,
            ));
        }
    }
    if let Some((selected, actions)) = system_menu {
        texts.push(FrameText::new(
            "System".to_owned(),
            Rect::new(496.0, 136.0, 288.0, 34.0),
            Color::rgba(0.88, 0.9, 0.96, 1.0),
            21_010,
        ));
        for (index, action) in actions.iter().enumerate() {
            let marker = if index == selected { "> " } else { "  " };
            texts.push(FrameText::new(
                format!("{marker}{}", action.label()),
                Rect::new(496.0, 190.0 + index as f32 * 58.0, 288.0, 30.0),
                Color::WHITE,
                21_020 + index as i32,
            ));
        }
    }
    texts
}

fn split_speaker_line(visible_text: &str) -> (Option<&str>, &str) {
    visible_text
        .split_once(": ")
        .map_or((None, visible_text), |(speaker, content)| {
            (Some(speaker), content)
        })
}

fn parse_value(value: &str) -> Value {
    match value {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => value
            .parse::<f64>()
            .map(Value::Number)
            .unwrap_or_else(|_| Value::Text(value.to_owned())),
    }
}

fn evaluate_condition(condition: &str, variables: &HashMap<String, Value>) -> bool {
    let condition = condition.trim();
    if let Some(name) = condition.strip_prefix('!') {
        return !truthy(variables.get(name.trim()));
    }

    for operator in ["==", "!=", ">=", "<=", ">", "<"] {
        if let Some((left, right)) = condition.split_once(operator) {
            return compare_values(
                variables.get(left.trim()),
                operator,
                &parse_value(right.trim().trim_matches('"')),
            );
        }
    }

    truthy(variables.get(condition))
}

fn compare_values(left: Option<&Value>, operator: &str, right: &Value) -> bool {
    let Some(left) = left else {
        return matches!(operator, "!=");
    };

    match operator {
        "==" => values_equal(left, right),
        "!=" => !values_equal(left, right),
        ">" => numeric_value(left)
            .zip(numeric_value(right))
            .is_some_and(|(l, r)| l > r),
        "<" => numeric_value(left)
            .zip(numeric_value(right))
            .is_some_and(|(l, r)| l < r),
        ">=" => numeric_value(left)
            .zip(numeric_value(right))
            .is_some_and(|(l, r)| l >= r),
        "<=" => numeric_value(left)
            .zip(numeric_value(right))
            .is_some_and(|(l, r)| l <= r),
        _ => false,
    }
}

fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Bool(left), Value::Bool(right)) => left == right,
        (Value::Number(left), Value::Number(right)) => (left - right).abs() < f64::EPSILON,
        (Value::Text(left), Value::Text(right)) => left == right,
        _ => false,
    }
}

fn numeric_value(value: &Value) -> Option<f64> {
    match value {
        Value::Number(value) => Some(*value),
        Value::Text(value) => value.parse().ok(),
        Value::Bool(_) => None,
    }
}

fn wrapped_index(index: usize, len: usize, delta: i32) -> usize {
    if len == 0 {
        return 0;
    }

    if delta >= 0 {
        (index + delta as usize) % len
    } else {
        let delta = delta.unsigned_abs() as usize % len;
        (index + len - delta) % len
    }
}

fn truthy(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(value)) => *value,
        Some(Value::Number(value)) => *value != 0.0,
        Some(Value::Text(value)) => !value.is_empty(),
        None => false,
    }
}

fn dialogue_key(
    script_entry: &str,
    command_position: usize,
    speaker: Option<&str>,
    content: &str,
) -> String {
    let speaker = speaker.unwrap_or("");
    let text = normalize_text_markup(content);
    format!("{script_entry}:{command_position}:{speaker}:{text}")
}

fn restored_dialogue_key(script_entry: &str, dialogue: &TextBlock) -> String {
    format!("{script_entry}:restored::{}", dialogue.raw)
}

fn sorted_read_dialogue_keys(read_dialogue_keys: &HashSet<String>) -> Vec<String> {
    let mut keys = read_dialogue_keys.iter().cloned().collect::<Vec<_>>();
    keys.sort();
    keys
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_advance_updates_scene_and_audio() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@bg file=\"school\"\n@playbgm file=\"theme\" loop=true\n# 艾琳\n你好")
            .unwrap();

        assert!(app.advance_script());
        assert_eq!(app.scene.background.as_ref().unwrap().texture_id, "school");

        assert!(app.advance_script());
        assert!(matches!(
            app.audio.bgm.current,
            Some(AudioSource::File { ref path, looping: true }) if path == "theme"
        ));

        assert!(app.advance_script());
        assert!(app.scene.dialogue.as_ref().unwrap().raw.contains("艾琳"));
        assert_eq!(
            app.scene.dialogue.as_ref().unwrap().reveal.revealed_chars,
            0
        );
    }

    #[test]
    fn bgm_commands_apply_fade_timing() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@playbgm file=\"theme\" loop=true fadein=1000\n@stopbgm fadeout=1000")
            .unwrap();

        assert!(app.advance_script());
        assert_eq!(app.audio.bgm.volume, 0.0);

        app.tick(500);
        assert_eq!(app.audio.bgm.volume, 0.5);

        app.tick(500);
        assert_eq!(app.audio.bgm.volume, 1.0);
        assert!(app.audio.bgm.current.is_some());

        assert!(app.advance_script());
        app.tick(500);
        assert_eq!(app.audio.bgm.volume, 0.5);
        assert!(app.audio.bgm.current.is_some());

        app.tick(500);
        assert!(app.audio.bgm.current.is_none());
        assert_eq!(app.audio.bgm.volume, 1.0);
    }

    #[test]
    fn user_settings_apply_audio_volumes() {
        let mut app = SuzuApp::new(GameConfig::default());
        let mut settings = UserSettings::default();
        settings.audio.master_volume = 0.8;
        settings.audio.bgm_volume = 0.6;
        settings.audio.voice_volume = 0.7;
        settings.audio.se_volume = 0.5;

        app.apply_user_settings(settings);

        assert_eq!(app.audio.master_volume, 0.8);
        assert_eq!(app.audio.bgm_volume, 0.6);
        assert_eq!(app.audio.voice_volume, 0.7);
        assert_eq!(app.audio.se_volume, 0.5);
    }

    #[test]
    fn user_settings_clamp_audio_volumes() {
        let mut app = SuzuApp::new(GameConfig::default());
        let mut settings = UserSettings::default();
        settings.audio.master_volume = 2.0;
        settings.audio.bgm_volume = -1.0;

        app.apply_user_settings(settings);

        assert_eq!(app.audio.master_volume, 1.0);
        assert_eq!(app.audio.bgm_volume, 0.0);
    }

    #[test]
    fn voice_commands_apply_to_voice_channel() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@playvoice file=\"voices/eileen_001.ogg\" fadein=1000\n@stopvoice fadeout=1000",
        )
        .unwrap();

        assert!(app.advance_script());
        assert_eq!(app.audio.voice.volume, 0.0);
        assert!(matches!(
            app.audio.voice.current,
            Some(AudioSource::File { ref path, looping: false }) if path == "voices/eileen_001.ogg"
        ));

        app.tick(500);
        assert_eq!(app.audio.voice.volume, 0.5);
        app.tick(500);
        assert_eq!(app.audio.voice.volume, 1.0);

        assert!(app.advance_script());
        app.tick(500);
        assert_eq!(app.audio.voice.volume, 0.5);
        assert!(app.audio.voice.current.is_some());

        app.tick(500);
        assert!(app.audio.voice.current.is_none());
        assert_eq!(app.audio.voice.volume, 1.0);
    }

    #[test]
    fn voice_command_cues_next_dialogue_line() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@voice file=\"voices/eileen_001.ogg\" fadein=200\n# 艾琳\n你好")
            .unwrap();

        assert!(app.advance_script());
        assert!(app.audio.voice.current.is_none());
        assert_eq!(
            app.pending_voice.as_ref().unwrap().file,
            "voices/eileen_001.ogg"
        );

        assert!(app.advance_script());
        assert_eq!(app.audio.voice.volume, 0.0);
        assert!(matches!(
            app.audio.voice.current,
            Some(AudioSource::File { ref path, looping: false }) if path == "voices/eileen_001.ogg"
        ));
        assert!(app
            .scene
            .dialogue
            .as_ref()
            .unwrap()
            .segments
            .iter()
            .any(|segment| matches!(
                segment,
                TextSegment::VoiceSync { char_index: 0, voice_file } if voice_file == "voices/eileen_001.ogg"
            )));
        assert!(app.pending_voice.is_none());
    }

    #[test]
    fn queued_voice_is_replaced_by_later_voice_command() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@voice file=\"voices/old.ogg\"\n@voice file=\"voices/new.ogg\"\n# 艾琳\n你好",
        )
        .unwrap();

        assert!(app.advance_script());
        assert!(app.advance_script());
        assert_eq!(app.pending_voice.as_ref().unwrap().file, "voices/new.ogg");

        assert!(app.advance_script());
        assert!(matches!(
            app.audio.voice.current,
            Some(AudioSource::File { ref path, looping: false }) if path == "voices/new.ogg"
        ));
    }

    #[test]
    fn save_state_preserves_voice_source() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@playvoice file=\"voices/eileen_001.ogg\"")
            .unwrap();

        assert!(app.advance_script());
        let state = app.capture_state();

        assert!(matches!(
            state.audio.voice,
            Some(AudioSource::File { ref path, looping: false }) if path == "voices/eileen_001.ogg"
        ));
    }

    #[test]
    fn registered_texture_is_loaded_when_script_references_it() {
        let mut path = std::env::temp_dir();
        path.push("suzu-app-bg-texture.png");
        let image = image::RgbaImage::from_raw(1, 1, vec![1, 2, 3, 255]).unwrap();
        image.save(&path).unwrap();

        let mut app = SuzuApp::new(GameConfig::default());
        app.register_texture("school", &path);
        app.load_script("@bg file=\"school\"").unwrap();

        assert!(app.advance_script());
        assert_eq!(app.scene_textures.len(), 1);
        assert_eq!(app.scene_textures[0].id, "school");
        assert_eq!(app.scene_textures[0].rgba, vec![1, 2, 3, 255]);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn animation_updates_sprite_state() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@char name=\"eileen\" pos=center\n@anim target=\"eileen\" type=zoom scale=1.25",
        )
        .unwrap();

        assert!(app.advance_script());
        assert!(app.advance_script());

        app.tick(500);

        assert_eq!(app.scene.characters[0].scale, Vec2::new(1.25, 1.25));
    }

    #[test]
    fn character_face_selects_expression_texture_and_keeps_name_target() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@char name=\"eileen\" face=\"happy\" pos=center\n@anim target=\"eileen\" type=move_to x=520 y=20 duration=0",
        )
        .unwrap();

        assert!(app.advance_script());
        assert_eq!(app.scene.characters[0].entity_id.as_deref(), Some("eileen"));
        assert_eq!(app.scene.characters[0].texture_id, "eileen_happy");

        assert!(app.advance_script());
        assert_eq!(app.scene.characters[0].position, Vec2::new(520.0, 20.0));
    }

    #[test]
    fn character_command_supports_custom_position() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@char name=\"eileen\" x=320 y=24 layer=4")
            .unwrap();

        assert!(app.advance_script());
        assert_eq!(app.scene.characters[0].position, Vec2::new(320.0, 24.0));
        assert_eq!(app.scene.characters[0].size, Vec2::new(360.0, 720.0));
        assert_eq!(app.scene.characters[0].z_index, 4);
    }

    #[test]
    fn character_command_supports_custom_size() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@char name=\"eileen\" width=420 height=680")
            .unwrap();

        assert!(app.advance_script());
        assert_eq!(app.scene.characters[0].size, Vec2::new(420.0, 680.0));
    }

    #[test]
    fn character_command_supports_horizontal_flip() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@char name=\"eileen\" flip=true").unwrap();

        assert!(app.advance_script());
        assert!(app.scene.characters[0].flip_x);

        let frame = app.update(0);
        let sprite = frame
            .sprites
            .iter()
            .find(|sprite| sprite.texture_id == "eileen")
            .unwrap();
        assert!(sprite.flip_x);
    }

    #[test]
    fn fade_animation_updates_character_opacity_immediately() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@char name=\"eileen\" pos=center\n@anim target=\"eileen\" type=fade opacity=0.25",
        )
        .unwrap();

        assert!(app.advance_script());
        assert!(app.advance_script());

        assert_eq!(app.scene.characters[0].opacity, 0.25);
    }

    #[test]
    fn repeated_character_command_updates_existing_layer() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@char name=\"eileen\" face=\"happy\" pos=center layer=10\n@anim target=\"eileen\" type=zoom scale=1.2\n@char name=\"eileen\" face=\"blush\" pos=right layer=12",
        )
        .unwrap();

        assert!(app.advance_script());
        assert!(app.advance_script());
        assert!(app.advance_script());

        assert_eq!(app.scene.characters.len(), 1);
        assert_eq!(app.scene.characters[0].texture_id, "eileen_blush");
        assert_eq!(app.scene.characters[0].position, Vec2::new(740.0, 0.0));
        assert_eq!(app.scene.characters[0].size, Vec2::new(360.0, 720.0));
        assert_eq!(app.scene.characters[0].scale, Vec2::new(1.2, 1.2));
        assert!(!app.scene.characters[0].flip_x);
        assert_eq!(app.scene.characters[0].z_index, 12);
    }

    #[test]
    fn repeated_character_command_updates_existing_flip() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@char name=\"eileen\"\n@char name=\"eileen\" flip=true")
            .unwrap();

        assert!(app.advance_script());
        assert!(app.advance_script());

        assert_eq!(app.scene.characters.len(), 1);
        assert!(app.scene.characters[0].flip_x);
    }

    #[test]
    fn repeated_character_command_updates_existing_size() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@char name=\"eileen\" width=360 height=720\n@char name=\"eileen\" width=420 height=680",
        )
        .unwrap();

        assert!(app.advance_script());
        assert!(app.advance_script());

        assert_eq!(app.scene.characters.len(), 1);
        assert_eq!(app.scene.characters[0].size, Vec2::new(420.0, 680.0));
    }

    #[test]
    fn hide_character_removes_layer_and_pending_animation() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@char name=\"eileen\" pos=center\n@anim target=\"eileen\" type=move_to x=560 y=40 duration=1000\n@hidechar name=\"eileen\"",
        )
        .unwrap();

        assert!(app.advance_script());
        assert!(app.advance_script());
        assert_eq!(app.scene.characters.len(), 1);
        assert_eq!(app.active_animations.len(), 1);

        assert!(app.advance_script());
        assert!(app.scene.characters.is_empty());
        assert!(app.active_animations.is_empty());
    }

    #[test]
    fn wait_command_pauses_script_then_resumes() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@wait time=500\n# N\nAfter").unwrap();

        app.advance_until_waiting();
        assert_eq!(app.wait_timer_ms, Some(500));
        assert!(app.scene.dialogue.is_none());

        app.tick(499);
        assert_eq!(app.wait_timer_ms, Some(1));
        assert!(app.scene.dialogue.is_none());

        app.tick(1);
        assert!(app.wait_timer_ms.is_none());
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: After");
    }

    #[test]
    fn confirm_does_not_skip_wait_command() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@wait time=500\n# N\nAfter").unwrap();

        app.advance_until_waiting();
        app.confirm();

        assert_eq!(app.wait_timer_ms, Some(500));
        assert!(app.scene.dialogue.is_none());
    }

    #[test]
    fn message_box_visibility_commands_affect_frame_output() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# N\nLine\n@hidemsg\n@showmsg").unwrap();

        app.advance_until_waiting();
        app.reveal_dialogue_now();

        let frame = app.update(0);
        assert!(frame
            .sprites
            .iter()
            .any(|sprite| sprite.texture_id == "dialogue"));
        assert!(frame.texts.iter().any(|text| text.content == "N"));
        assert!(frame.texts.iter().any(|text| text.content == "Line"));

        assert!(app.advance_script());
        let frame = app.update(0);
        assert!(!frame
            .sprites
            .iter()
            .any(|sprite| sprite.texture_id == "dialogue"));
        assert!(!frame.texts.iter().any(|text| text.content == "N"));
        assert!(!frame.texts.iter().any(|text| text.content == "Line"));

        assert!(app.advance_script());
        let frame = app.update(0);
        assert!(frame
            .sprites
            .iter()
            .any(|sprite| sprite.texture_id == "dialogue"));
        assert!(frame.texts.iter().any(|text| text.content == "N"));
        assert!(frame.texts.iter().any(|text| text.content == "Line"));
    }

    #[test]
    fn save_state_preserves_message_box_visibility() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# N\nLine\n@hidemsg").unwrap();

        app.advance_until_waiting();
        app.reveal_dialogue_now();
        assert!(app.advance_script());

        let state = app.capture_state();
        assert!(!state.scene.message_box_visible);

        let mut restored = SuzuApp::new(GameConfig::default());
        restored.restore_state(state);
        assert!(!restored.scene.message_box_visible);
        assert_eq!(restored.scene.dialogue.as_ref().unwrap().raw, "N: Line");
    }

    #[test]
    fn animation_interpolates_over_time() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@char name=\"eileen\" pos=center\n@anim target=\"eileen\" type=move_to x=560 y=40 duration=1000",
        )
        .unwrap();

        assert!(app.advance_script());
        assert!(app.advance_script());

        app.tick(500);
        let halfway = app.scene.characters[0].position;
        assert!(halfway.x > 460.0 && halfway.x < 560.0);
        assert!(halfway.y > 0.0 && halfway.y < 40.0);

        app.tick(500);
        assert_eq!(app.scene.characters[0].position, Vec2::new(560.0, 40.0));
    }

    #[test]
    fn fade_animation_interpolates_over_time() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@char name=\"eileen\" pos=center\n@anim target=\"eileen\" type=fade opacity=0 duration=1000",
        )
        .unwrap();

        assert!(app.advance_script());
        assert!(app.advance_script());

        app.tick(500);
        assert!(app.scene.characters[0].opacity > 0.0);
        assert!(app.scene.characters[0].opacity < 1.0);

        app.tick(500);
        assert_eq!(app.scene.characters[0].opacity, 0.0);
    }

    #[test]
    fn background_crossfade_interpolates_opacity() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@bg file=\"school\"\n@bg file=\"rooftop\" time=1000 method=crossfade")
            .unwrap();

        assert!(app.advance_script());
        assert_eq!(app.scene.background.as_ref().unwrap().texture_id, "school");

        assert!(app.advance_script());
        assert_eq!(app.scene.background.as_ref().unwrap().texture_id, "rooftop");
        assert!(app.scene.outgoing_background.is_some());

        app.tick(500);
        let incoming = app.scene.background.as_ref().unwrap().opacity;
        let outgoing = app.scene.outgoing_background.as_ref().unwrap().opacity;
        assert!(incoming > 0.0 && incoming < 1.0);
        assert!(outgoing > 0.0 && outgoing < 1.0);

        app.tick(500);
        assert_eq!(app.scene.background.as_ref().unwrap().opacity, 1.0);
        assert!(app.scene.outgoing_background.is_none());
    }

    #[test]
    fn background_fade_through_color_uses_overlay_and_delays_incoming_opacity() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@bg file=\"school\"\n@bg file=\"rooftop\" time=1000 method=fade_through_color color=#112233",
        )
        .unwrap();

        assert!(app.advance_script());
        assert!(app.advance_script());

        app.tick(250);
        assert_eq!(app.scene.background.as_ref().unwrap().opacity, 0.0);
        assert!(app.scene.outgoing_background.as_ref().unwrap().opacity < 1.0);

        let frame = app.update(0);
        let overlay = frame
            .sprites
            .iter()
            .find(|sprite| sprite.texture_id == "bg_transition_color")
            .unwrap();
        assert_eq!(
            overlay.tint,
            Color::rgba(
                0x11 as f32 / 255.0,
                0x22 as f32 / 255.0,
                0x33 as f32 / 255.0,
                1.0
            )
        );
        assert!(overlay.opacity > 0.0);

        app.tick(750);
        assert_eq!(app.scene.background.as_ref().unwrap().opacity, 1.0);
        assert!(app.scene.outgoing_background.is_none());
        let frame = app.update(0);
        assert!(!frame
            .sprites
            .iter()
            .any(|sprite| sprite.texture_id == "bg_transition_color"));
    }

    #[test]
    fn flash_effect_adds_fading_overlay() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@fx type=flash color=#FF0000 duration=1000\n# N\nFlash")
            .unwrap();

        app.advance_until_waiting();
        let frame = app.update(0);
        let flash = frame
            .sprites
            .iter()
            .find(|sprite| sprite.texture_id == "fx_flash")
            .unwrap();
        assert_eq!(flash.tint, Color::rgba(1.0, 0.0, 0.0, 1.0));
        assert_eq!(flash.opacity, 1.0);

        let frame = app.update(500);
        let flash = frame
            .sprites
            .iter()
            .find(|sprite| sprite.texture_id == "fx_flash")
            .unwrap();
        assert!(flash.opacity > 0.0 && flash.opacity < 1.0);

        let frame = app.update(500);
        assert!(!frame
            .sprites
            .iter()
            .any(|sprite| sprite.texture_id == "fx_flash"));
    }

    #[test]
    fn quake_effect_offsets_frame_layers_temporarily() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@bg file=\"school\"\n@fx type=quake intensity=12 duration=1000\n# N\nShake",
        )
        .unwrap();

        app.advance_until_waiting();
        let frame = app.update(128);
        let background = frame
            .sprites
            .iter()
            .find(|sprite| sprite.texture_id == "school")
            .unwrap();
        assert_ne!(background.bounds.origin, Vec2::ZERO);

        let frame = app.update(1000);
        let background = frame
            .sprites
            .iter()
            .find(|sprite| sprite.texture_id == "school")
            .unwrap();
        assert_eq!(background.bounds.origin, Vec2::ZERO);
    }

    #[test]
    fn dialogue_reveals_over_time() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# 艾琳\n你好世界").unwrap();

        assert!(app.advance_script());
        assert_eq!(app.scene.dialogue.as_ref().unwrap().visible_text(), "");

        app.tick(100);
        assert!(!app
            .scene
            .dialogue
            .as_ref()
            .unwrap()
            .visible_text()
            .is_empty());
        assert!(!app.scene.dialogue.as_ref().unwrap().reveal.is_complete());

        app.reveal_dialogue_now();
        assert!(app.scene.dialogue.as_ref().unwrap().reveal.is_complete());
        assert_eq!(
            app.scene.dialogue.as_ref().unwrap().visible_text(),
            app.scene.dialogue.as_ref().unwrap().raw
        );
    }

    #[test]
    fn user_settings_apply_dialogue_reveal_speed() {
        let mut app = SuzuApp::new(GameConfig::default());
        let mut settings = UserSettings::default();
        settings.text.speed_chars_per_second = 120.0;
        app.apply_user_settings(settings);
        app.load_script("# 艾琳\n你好").unwrap();

        app.advance_until_waiting();

        assert_eq!(
            app.scene
                .dialogue
                .as_ref()
                .unwrap()
                .reveal
                .speed_chars_per_second,
            120.0
        );
    }

    #[test]
    fn auto_mode_can_be_toggled() {
        let mut app = SuzuApp::new(GameConfig::default());

        assert!(!app.auto_mode());
        app.toggle_auto_mode();
        assert!(app.auto_mode());
        app.set_auto_mode(false);
        assert!(!app.auto_mode());
    }

    #[test]
    fn auto_mode_advances_after_dialogue_delay() {
        let mut app = SuzuApp::new(GameConfig::default());
        let mut settings = UserSettings::default();
        settings.text.auto_advance_delay_ms = 500;
        app.apply_user_settings(settings);
        app.load_script("# N\nFirst\n# N\nSecond").unwrap();

        app.advance_until_waiting();
        app.reveal_dialogue_now();
        app.set_auto_mode(true);

        app.tick(499);
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: First");

        app.tick(1);
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: Second");
    }

    #[test]
    fn auto_mode_does_not_confirm_choices() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@choice \"A\" goto=a\n*a\n# N\nA").unwrap();

        app.advance_until_waiting();
        app.set_auto_mode(true);
        app.tick(5000);

        assert!(app.scene.choice.is_some());
        assert!(app.scene.dialogue.is_none());
    }

    #[test]
    fn completed_dialogue_is_marked_as_read() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# N\nFirst").unwrap();

        app.advance_until_waiting();
        assert!(!app.is_current_dialogue_read());

        app.reveal_dialogue_now();
        assert!(app.is_current_dialogue_read());
    }

    #[test]
    fn skip_mode_advances_read_dialogue_and_stops_at_unread() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# N\nFirst\n# N\nSecond").unwrap();

        app.advance_until_waiting();
        app.reveal_dialogue_now();
        assert!(app.is_current_dialogue_read());

        app.load_script("# N\nFirst\n# N\nSecond").unwrap();
        app.advance_until_waiting();
        app.set_skip_mode(true);
        app.tick(0);

        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: Second");
        assert!(!app.is_current_dialogue_read());
        assert!(app.skip_mode());
    }

    #[test]
    fn skip_mode_stops_at_choices() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@choice \"A\" goto=a\n*a\n# N\nA").unwrap();

        app.advance_until_waiting();
        app.set_skip_mode(true);
        app.tick(0);

        assert!(app.scene.choice.is_some());
        assert!(!app.skip_mode());
    }

    #[test]
    fn save_state_preserves_read_dialogue_keys() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# N\nFirst").unwrap();
        app.advance_until_waiting();
        app.reveal_dialogue_now();

        let state = app.capture_state();
        assert_eq!(state.read_dialogue_keys.len(), 1);

        let mut restored = SuzuApp::new(GameConfig::default());
        restored.restore_state(state);
        assert_eq!(restored.read_dialogue_keys.len(), 1);
    }

    #[test]
    fn save_slot_can_store_thumbnail() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# N\nFirst").unwrap();
        app.advance_until_waiting();

        let thumbnail = SaveThumbnail::new(1, 1, vec![1, 2, 3, 255]).unwrap();
        assert!(app.save_slot_with_thumbnail(0, thumbnail));

        let saved = app.saves.load_slot(0).unwrap();
        let thumbnail = saved.metadata.thumbnail.as_ref().unwrap();
        assert_eq!(thumbnail.width, 1);
        assert_eq!(thumbnail.height, 1);
        assert_eq!(thumbnail.rgba, [1, 2, 3, 255]);
    }

    #[test]
    fn history_overlay_scrolls_and_renders_entries() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# N\nFirst\n# N\nSecond").unwrap();

        app.advance_until_waiting();
        app.reveal_dialogue_now();
        app.confirm();
        app.reveal_dialogue_now();
        app.open_history();

        assert!(app.history_visible());
        assert_eq!(app.visible_history_entries(1)[0].text, "Second");
        app.scroll_history(1);
        assert_eq!(app.visible_history_entries(1)[0].text, "First");

        let frame = app.update(0);
        assert!(frame
            .sprites
            .iter()
            .any(|sprite| sprite.texture_id == "history_backlog"));
        assert!(frame
            .texts
            .iter()
            .any(|text| text.content.contains("First")));
    }

    #[test]
    fn history_voice_replay_uses_entry_voice_file() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@voice file=\"voices/line.ogg\"\n# N\nVoiced")
            .unwrap();

        app.advance_until_waiting();
        app.reveal_dialogue_now();
        app.open_history();

        assert!(app.replay_history_voice(0));
        assert!(matches!(
            app.audio.voice.current,
            Some(AudioSource::File { ref path, looping: false }) if path == "voices/line.ogg"
        ));
    }

    #[test]
    fn cancel_input_closes_history_overlay() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.open_history();

        app.handle_input_event(InputEvent::Cancel);
        app.tick(0);

        assert!(!app.history_visible());
    }

    #[test]
    fn cancel_input_opens_system_menu_and_confirm_activates_history() {
        let mut app = SuzuApp::new(GameConfig::default());

        app.handle_input_event(InputEvent::Cancel);
        app.tick(0);
        assert!(app.system_menu_visible());

        app.move_system_menu_selection(3);
        assert_eq!(app.selected_system_menu_action(), SystemMenuAction::History);
        app.handle_input_event(InputEvent::Confirm);
        app.tick(0);

        assert!(!app.system_menu_visible());
        assert!(app.history_visible());
    }

    #[test]
    fn system_menu_save_and_load_use_slot_zero() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# N\nFirst\n# N\nSecond").unwrap();
        app.advance_until_waiting();
        app.reveal_dialogue_now();

        app.activate_system_menu_action(SystemMenuAction::Save);
        app.confirm();
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: Second");

        app.activate_system_menu_action(SystemMenuAction::Load);
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: First");
    }

    #[test]
    fn system_menu_quit_sets_request_flag() {
        let mut app = SuzuApp::new(GameConfig::default());

        app.open_system_menu();
        app.activate_system_menu_action(SystemMenuAction::Quit);

        assert!(app.quit_requested());
        assert!(!app.system_menu_visible());
    }

    #[test]
    fn title_screen_waits_until_start_is_selected() {
        let mut app = SuzuApp::new(title_config());
        app.load_script("# N\nFirst").unwrap();

        app.tick(0);
        assert!(app.title_screen_visible());
        assert!(app.scene.dialogue.is_none());

        app.handle_input_event(InputEvent::Confirm);
        app.tick(0);

        assert!(!app.title_screen_visible());
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: First");
    }

    #[test]
    fn title_menu_selection_wraps_and_quit_sets_request_flag() {
        let mut app = SuzuApp::new(title_config());
        app.load_script("# N\nFirst").unwrap();

        app.handle_input_event(InputEvent::MoveSelection { delta: -1 });
        app.tick(0);
        assert_eq!(app.selected_title_menu_action(), TitleMenuAction::Quit);

        app.handle_input_event(InputEvent::Confirm);
        app.tick(0);

        assert!(app.quit_requested());
        assert!(!app.title_screen_visible());
    }

    #[test]
    fn return_title_resets_runtime_and_shows_title_screen() {
        let mut app = SuzuApp::new(title_config());
        app.load_script("# N\nFirst\n# N\nSecond").unwrap();
        app.start_game();
        app.reveal_dialogue_now();
        app.confirm();

        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: Second");

        app.activate_system_menu_action(SystemMenuAction::ReturnTitle);

        assert!(app.title_screen_visible());
        assert!(app.scene.dialogue.is_none());
        assert!(app.history.is_empty());
    }

    #[test]
    fn dialogue_control_tags_are_not_displayed() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# 艾琳\n你好[l][r]下一行").unwrap();

        app.advance_until_waiting();
        app.reveal_dialogue_now();

        assert_eq!(
            app.scene.dialogue.as_ref().unwrap().raw,
            "艾琳: 你好\n下一行"
        );
        assert_eq!(app.history[0].text, "你好\n下一行");
    }

    #[test]
    fn dialogue_wait_tag_pauses_until_confirmed() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# 艾琳\n前半[l]后半\n# 艾琳\n下一句")
            .unwrap();

        app.advance_until_waiting();
        app.tick(1000);

        assert_eq!(
            app.scene.dialogue.as_ref().unwrap().visible_text(),
            "艾琳: 前半"
        );
        assert!(!app.scene.dialogue.as_ref().unwrap().reveal.is_complete());

        app.handle_input_event(InputEvent::Confirm);
        app.tick(0);
        assert_eq!(
            app.scene.dialogue.as_ref().unwrap().visible_text(),
            "艾琳: 前半"
        );

        app.tick(1000);
        assert_eq!(
            app.scene.dialogue.as_ref().unwrap().visible_text(),
            "艾琳: 前半后半"
        );

        app.handle_input_event(InputEvent::Confirm);
        app.tick(0);
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 下一句");
    }

    #[test]
    fn confirm_reveals_then_advances_script() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# 艾琳\n第一句\n# 艾琳\n第二句").unwrap();
        app.advance_until_waiting();

        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 第一句");
        assert!(!app.scene.dialogue.as_ref().unwrap().reveal.is_complete());

        app.handle_input_event(InputEvent::Confirm);
        app.tick(0);
        assert!(app.scene.dialogue.as_ref().unwrap().reveal.is_complete());
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 第一句");

        app.handle_input_event(InputEvent::Confirm);
        app.tick(0);
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 第二句");
        assert!(!app.scene.dialogue.as_ref().unwrap().reveal.is_complete());
    }

    #[test]
    fn choice_waits_and_jumps_to_selected_label() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@choice \"教室\" goto=classroom\n@choice \"天台\" goto=roof\n*classroom\n# 艾琳\n教室\n*roof\n# 艾琳\n天台",
        )
        .unwrap();

        app.advance_until_waiting();
        assert_eq!(app.scene.choice.as_ref().unwrap().options.len(), 2);

        app.handle_input_event(InputEvent::Scroll { delta: -1.0 });
        app.tick(0);
        assert_eq!(app.scene.choice.as_ref().unwrap().selected_index, 1);

        app.handle_input_event(InputEvent::Confirm);
        app.tick(0);
        assert!(app.scene.choice.is_none());
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 天台");
    }

    #[test]
    fn keyboard_selection_moves_choice() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@choice \"A\" goto=a\n@choice \"B\" goto=b\n*a\n# N\nA\n*b\n# N\nB")
            .unwrap();

        app.advance_until_waiting();
        app.handle_input_event(InputEvent::MoveSelection { delta: 1 });
        app.tick(0);
        assert_eq!(app.scene.choice.as_ref().unwrap().selected_index, 1);

        app.handle_input_event(InputEvent::MoveSelection { delta: -1 });
        app.tick(0);
        assert_eq!(app.scene.choice.as_ref().unwrap().selected_index, 0);
    }

    #[test]
    fn conditional_choices_filter_by_variables() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script(
            "@set name=affection_eileen value=52\n@choice \"普通路线\" goto=normal\n@choice \"艾琳路线\" goto=eileen cond=affection_eileen>=50\n*normal\n# N\n普通\n*eileen\n# N\n艾琳",
        )
        .unwrap();

        app.advance_until_waiting();
        assert_eq!(app.scene.choice.as_ref().unwrap().options.len(), 2);
        app.handle_input_event(InputEvent::MoveSelection { delta: 1 });
        app.handle_input_event(InputEvent::Confirm);
        app.tick(0);

        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: 艾琳");
    }

    #[test]
    fn false_conditional_choice_does_not_block_script() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@set name=seen_secret value=false\n@choice \"秘密\" goto=secret cond=seen_secret\n# N\n继续\n*secret\n# N\n秘密")
            .unwrap();

        app.advance_until_waiting();

        assert!(app.scene.choice.is_none());
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: 继续");
    }

    #[test]
    fn if_block_inserts_commands_when_condition_is_true() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@set name=affection_eileen value=52\n@if cond=affection_eileen>=50\n# 艾琳\n条件成立\n@endif\n# 艾琳\n结束")
            .unwrap();

        app.advance_until_waiting();

        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 条件成立");
        app.reveal_dialogue_now();
        app.confirm();
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 结束");
    }

    #[test]
    fn if_block_skips_commands_when_condition_is_false() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@set name=affection_eileen value=10\n@if cond=affection_eileen>=50\n# 艾琳\n条件成立\n@endif\n# 艾琳\n结束")
            .unwrap();

        app.advance_until_waiting();

        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 结束");
    }

    #[test]
    fn if_else_block_runs_else_when_condition_is_false() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@set name=affection_eileen value=10\n@if cond=affection_eileen>=50\n# 艾琳\n真\n@else\n# 艾琳\n假\n@endif")
            .unwrap();

        app.advance_until_waiting();

        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 假");
    }

    #[test]
    fn call_and_return_resume_after_subroutine() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@call goto=common\n# N\n主线\n*common\n# N\n共通\n@return\n# N\n不会重复")
            .unwrap();

        app.advance_until_waiting();
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: 共通");

        app.reveal_dialogue_now();
        app.confirm();
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: 主线");
    }

    #[test]
    fn save_state_preserves_call_stack() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@call goto=common\n# N\n主线\n*common\n# N\n共通\n@return")
            .unwrap();

        assert!(app.advance_script());
        let state = app.capture_state();

        assert_eq!(state.script.call_stack.len(), 1);
        let mut restored = SuzuApp::new(GameConfig::default());
        restored.restore_state(state);
        restored.advance_until_waiting();
        assert_eq!(restored.scene.dialogue.as_ref().unwrap().raw, "N: 共通");
    }

    #[test]
    fn frame_contains_dialogue_and_choice_texts() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# 艾琳\n你好\n@choice \"A\" goto=a\n*a\n# 艾琳\nA")
            .unwrap();
        app.advance_until_waiting();
        app.reveal_dialogue_now();
        app.confirm();

        let frame = app.update(0);
        assert!(frame.texts.iter().any(|text| text.content == "A"));
    }

    #[test]
    fn autosave_command_captures_scene_snapshot() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("@savename text=\"第一章\"\n@bg file=\"school\"\n@autosave")
            .unwrap();

        app.advance_until_waiting();
        let autosave = app.saves.autosave().unwrap();

        assert_eq!(autosave.metadata.title, "第一章");
        assert_eq!(
            autosave.scene.background.as_ref().unwrap().texture_id,
            "school"
        );
        assert_eq!(autosave.script.line_number, 3);
        assert_eq!(autosave.script.pending_commands.len(), 3);
    }

    #[test]
    fn save_slot_restore_resumes_from_script_position() {
        let mut app = SuzuApp::new(GameConfig::default());
        app.load_script("# 艾琳\n第一句\n# 艾琳\n第二句").unwrap();
        app.advance_until_waiting();
        app.reveal_dialogue_now();

        assert!(app.save_slot(0));
        app.confirm();
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 第二句");

        assert!(app.load_slot(0));
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 第一句");

        app.confirm();
        assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 第二句");
    }

    fn title_config() -> GameConfig {
        GameConfig {
            title_screen: crate::config::TitleScreenConfig {
                enabled: true,
                title: "Title".to_owned(),
                subtitle: "Subtitle".to_owned(),
            },
            ..GameConfig::default()
        }
    }
}
