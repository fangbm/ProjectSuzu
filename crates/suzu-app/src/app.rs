pub(super) use std::{
    collections::{HashMap, HashSet},
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) use suzu_asset::AssetManager;
pub(super) use suzu_audio::AudioSource;
pub(super) use suzu_audio::AudioSystem;
pub(super) use suzu_core::{Color, Rect, Vec2};
pub(super) use suzu_input::{InputEvent, InputState};
pub(super) use suzu_platform::{
    DesktopApp, DesktopFrame, DesktopInputEvent, FrameBlendMode, FrameSprite, FrameText,
    FrameTexture,
};
pub(super) use suzu_render::{BlendMode, Easing, Renderer, SpriteLayer, Tween};
pub(super) use suzu_save::{
    AudioState, ChoiceStateSnapshot, GameState, HistoryEntry, SaveManager, SaveMetadata,
    SaveThumbnail, SceneState, ScriptState, Value, SAVE_FORMAT_VERSION,
};
pub(super) use suzu_script::{
    compile_script, AnimationKind, Command, CommandQueue, Position, Transition, VisualEffect,
};
pub(super) use suzu_text::{normalize_text_markup, TextBlock, TextSegment};

pub(super) use crate::{scene::ChoiceState, GameConfig, Scene, UserSettings};

mod animation;
mod command_handler;
mod effects;
mod frame_build;
mod history;
mod input_handler;
mod runtime;
mod save_flow;
mod system_menu;
mod title_screen;

use animation::{
    character_position, character_texture_id, sprite, upsert_character, ActiveAnimation,
};
use command_handler::{restored_dialogue_key, sorted_read_dialogue_keys, wrapped_index};
use effects::ActiveVisualEffect;
use runtime::BackgroundTransitionKind;
use runtime::{BackgroundTransition, PendingVoice};

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
const TITLE_LOAD_SLOT_COUNT: usize = 5;
const TITLE_LOAD_ENTRY_COUNT: usize = TITLE_LOAD_SLOT_COUNT + 2;
const TITLE_SETTINGS_ENTRY_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TitleScreenMode {
    Main,
    Load,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleMenuAction {
    Start,
    Continue,
    Load,
    Settings,
    Quit,
}

impl TitleMenuAction {
    fn label(self, labels: &crate::config::TitleScreenLabels) -> &str {
        match self {
            Self::Start => &labels.start,
            Self::Continue => &labels.continue_game,
            Self::Load => &labels.load,
            Self::Settings => &labels.settings,
            Self::Quit => &labels.quit,
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
    title_screen_mode: TitleScreenMode,
    title_menu_selected: usize,
    title_submenu_selected: usize,
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
            title_screen_mode: TitleScreenMode::Main,
            title_menu_selected: 0,
            title_submenu_selected: 0,
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

    pub fn load_script_asset(&mut self, id: impl Into<suzu_asset::AssetId>) -> anyhow::Result<()> {
        let id = id.into();
        let bytes = self.assets.load_asset_bytes(id.clone())?;
        let source = String::from_utf8(bytes)
            .map_err(|err| anyhow::anyhow!("script asset `{}` is not UTF-8: {err}", id.0))?;
        self.load_script(&source)
            .map_err(|err| anyhow::anyhow!("failed to compile script asset `{}`: {err}", id.0))
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

    pub fn register_package_file(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<usize> {
        self.assets.register_package_file(path)
    }

    pub fn register_xp3_file(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<usize> {
        self.assets.register_xp3_file(path)
    }

    pub fn register_xp3_file_with_options(
        &mut self,
        path: impl AsRef<std::path::Path>,
        options: suzu_asset::Xp3Options,
    ) -> anyhow::Result<usize> {
        self.assets.register_xp3_file_with_options(path, options)
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
}

#[cfg(test)]
mod tests;
