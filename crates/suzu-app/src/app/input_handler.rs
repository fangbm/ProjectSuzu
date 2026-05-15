use super::*;

impl SuzuApp {
    pub(super) fn process_input(&mut self) {
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
}
