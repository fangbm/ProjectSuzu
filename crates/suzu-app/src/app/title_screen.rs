use super::*;

impl SuzuApp {
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
}
