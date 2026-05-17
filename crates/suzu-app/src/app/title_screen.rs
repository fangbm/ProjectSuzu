use super::*;

impl SuzuApp {
    pub fn show_title_screen(&mut self) {
        self.reset_runtime_to_script_start();
        self.title_screen_visible = true;
        self.title_screen_mode = TitleScreenMode::Main;
        self.title_submenu_selected = 0;
    }

    pub fn title_screen_visible(&self) -> bool {
        self.title_screen_visible
    }

    pub fn selected_title_menu_action(&self) -> TitleMenuAction {
        TITLE_MENU_ACTIONS[self.title_menu_selected]
    }

    pub fn move_title_menu_selection(&mut self, delta: i32) {
        match self.title_screen_mode {
            TitleScreenMode::Main => {
                self.title_menu_selected =
                    wrapped_index(self.title_menu_selected, TITLE_MENU_ACTIONS.len(), delta);
            }
            TitleScreenMode::Load => {
                self.title_submenu_selected =
                    wrapped_index(self.title_submenu_selected, TITLE_LOAD_ENTRY_COUNT, delta);
            }
            TitleScreenMode::Settings => {
                self.title_submenu_selected = wrapped_index(
                    self.title_submenu_selected,
                    TITLE_SETTINGS_ENTRY_COUNT,
                    delta,
                );
            }
        }
    }

    pub fn start_game(&mut self) {
        self.reset_runtime_to_script_start();
        self.title_screen_visible = false;
        self.title_screen_mode = TitleScreenMode::Main;
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
                self.title_screen_mode = TitleScreenMode::Load;
                self.title_submenu_selected = first_available_load_entry(&self.saves).unwrap_or(0);
            }
            TitleMenuAction::Settings => {
                self.title_screen_mode = TitleScreenMode::Settings;
                self.title_submenu_selected = 0;
            }
            TitleMenuAction::Quit => {
                self.quit_requested = true;
                self.title_screen_visible = false;
            }
        }
    }

    pub(super) fn handle_title_input(&mut self, event: InputEvent) {
        match self.title_screen_mode {
            TitleScreenMode::Main => self.handle_title_main_input(event),
            TitleScreenMode::Load => self.handle_title_load_input(event),
            TitleScreenMode::Settings => self.handle_title_settings_input(event),
        }
    }

    fn handle_title_main_input(&mut self, event: InputEvent) {
        match event {
            InputEvent::Cancel => self.activate_title_menu_action(TitleMenuAction::Quit),
            InputEvent::Confirm => {
                self.activate_title_menu_selection();
            }
            InputEvent::PointerDown { position } | InputEvent::TouchStart { position, .. } => {
                if let Some(index) = title_menu_index_at(position, TITLE_MENU_ACTIONS.len()) {
                    self.title_menu_selected = index;
                    self.activate_title_menu_selection();
                }
            }
            InputEvent::PointerMove { position } => {
                if let Some(index) = title_menu_index_at(position, TITLE_MENU_ACTIONS.len()) {
                    self.title_menu_selected = index;
                }
            }
            InputEvent::Scroll { delta } => {
                self.move_title_menu_selection(if delta < 0.0 { 1 } else { -1 });
            }
            InputEvent::MoveSelection { delta } => self.move_title_menu_selection(delta),
            InputEvent::PointerUp { .. }
            | InputEvent::TouchMove { .. }
            | InputEvent::TouchEnd { .. } => {}
        }
    }

    fn handle_title_load_input(&mut self, event: InputEvent) {
        match event {
            InputEvent::Cancel => self.close_title_submenu(),
            InputEvent::Confirm => self.activate_title_load_selection(),
            InputEvent::PointerDown { position } | InputEvent::TouchStart { position, .. } => {
                if let Some(index) = title_menu_index_at(position, TITLE_LOAD_ENTRY_COUNT) {
                    self.title_submenu_selected = index;
                    self.activate_title_load_selection();
                }
            }
            InputEvent::PointerMove { position } => {
                if let Some(index) = title_menu_index_at(position, TITLE_LOAD_ENTRY_COUNT) {
                    self.title_submenu_selected = index;
                }
            }
            InputEvent::Scroll { delta } => {
                self.move_title_menu_selection(if delta < 0.0 { 1 } else { -1 });
            }
            InputEvent::MoveSelection { delta } => self.move_title_menu_selection(delta),
            InputEvent::PointerUp { .. }
            | InputEvent::TouchMove { .. }
            | InputEvent::TouchEnd { .. } => {}
        }
    }

    fn handle_title_settings_input(&mut self, event: InputEvent) {
        match event {
            InputEvent::Cancel => self.close_title_submenu(),
            InputEvent::Confirm => self.activate_title_settings_selection(),
            InputEvent::PointerDown { position } | InputEvent::TouchStart { position, .. } => {
                if let Some(index) = title_menu_index_at(position, TITLE_SETTINGS_ENTRY_COUNT) {
                    self.title_submenu_selected = index;
                    self.activate_title_settings_selection();
                }
            }
            InputEvent::PointerMove { position } => {
                if let Some(index) = title_menu_index_at(position, TITLE_SETTINGS_ENTRY_COUNT) {
                    self.title_submenu_selected = index;
                }
            }
            InputEvent::Scroll { delta } => {
                self.move_title_menu_selection(if delta < 0.0 { 1 } else { -1 });
            }
            InputEvent::MoveSelection { delta } => self.move_title_menu_selection(delta),
            InputEvent::PointerUp { .. }
            | InputEvent::TouchMove { .. }
            | InputEvent::TouchEnd { .. } => {}
        }
    }

    fn activate_title_load_selection(&mut self) {
        match self.title_submenu_selected {
            0 => {
                if let Some(state) = self.saves.autosave().cloned() {
                    self.restore_state(state);
                }
            }
            index @ 1..=TITLE_LOAD_SLOT_COUNT => {
                let _ = self.load_slot(index - 1);
            }
            _ => self.close_title_submenu(),
        }
    }

    fn activate_title_settings_selection(&mut self) {
        match self.title_submenu_selected {
            0 => {
                self.settings.text.speed_chars_per_second = next_value(
                    self.settings.text.speed_chars_per_second,
                    &[30.0, 60.0, 90.0, 120.0],
                );
            }
            1 => {
                self.settings.text.auto_advance_delay_ms = next_u32_value(
                    self.settings.text.auto_advance_delay_ms,
                    &[800, 1200, 1600, 2200],
                );
            }
            2 => {
                let mut settings = self.settings.clone();
                settings.audio.master_volume =
                    next_value(settings.audio.master_volume, &[0.5, 0.75, 1.0]);
                self.apply_user_settings(settings);
            }
            _ => self.close_title_submenu(),
        }
    }

    fn close_title_submenu(&mut self) {
        self.title_screen_mode = TitleScreenMode::Main;
        self.title_submenu_selected = 0;
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
        self.title_screen_mode = TitleScreenMode::Main;
        self.title_submenu_selected = 0;
    }
}

pub(super) fn title_menu_item_bounds(index: usize) -> Rect {
    Rect::new(752.0, 252.0 + index as f32 * 58.0, 304.0, 42.0)
}

fn title_menu_index_at(position: Vec2, count: usize) -> Option<usize> {
    (0..count).find(|index| title_menu_item_bounds(*index).contains(position))
}

fn first_available_load_entry(saves: &SaveManager) -> Option<usize> {
    if saves.autosave().is_some() {
        return Some(0);
    }

    (0..TITLE_LOAD_SLOT_COUNT)
        .find(|slot| saves.load_slot(*slot).is_some())
        .map(|slot| slot + 1)
}

fn next_value(current: f32, values: &[f32]) -> f32 {
    let index = values
        .iter()
        .position(|value| (*value - current).abs() < f32::EPSILON)
        .unwrap_or_else(|| {
            values
                .iter()
                .position(|value| *value >= current)
                .unwrap_or(0)
        });
    values[(index + 1) % values.len()]
}

fn next_u32_value(current: u32, values: &[u32]) -> u32 {
    let index = values
        .iter()
        .position(|value| *value == current)
        .unwrap_or_else(|| {
            values
                .iter()
                .position(|value| *value >= current)
                .unwrap_or(0)
        });
    values[(index + 1) % values.len()]
}
