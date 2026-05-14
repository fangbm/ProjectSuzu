use super::*;

impl SuzuApp {
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
}
