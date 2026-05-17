use super::super::*;

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
fn title_load_menu_can_restore_a_saved_slot() {
    let mut app = SuzuApp::new(title_config());
    app.load_script("# N\nFirst\n# N\nSecond").unwrap();
    app.start_game();
    app.reveal_dialogue_now();
    app.confirm();
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: Second");
    assert!(app.save_slot(0));

    app.show_title_screen();
    app.activate_title_menu_action(TitleMenuAction::Load);

    assert_eq!(app.title_screen_mode, TitleScreenMode::Load);
    assert_eq!(app.title_submenu_selected, 1);

    app.handle_input_event(InputEvent::Confirm);
    app.tick(0);

    assert!(!app.title_screen_visible());
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: Second");
}

#[test]
fn title_settings_menu_cycles_runtime_settings() {
    let mut app = SuzuApp::new(title_config());
    app.load_script("# N\nFirst").unwrap();
    assert_eq!(app.settings.text.speed_chars_per_second, 60.0);

    app.activate_title_menu_action(TitleMenuAction::Settings);
    assert_eq!(app.title_screen_mode, TitleScreenMode::Settings);

    app.handle_input_event(InputEvent::Confirm);
    app.tick(0);

    assert_eq!(app.settings.text.speed_chars_per_second, 90.0);
}

#[test]
fn title_screen_mouse_click_activates_hit_menu_item() {
    let mut app = SuzuApp::new(title_config());
    app.load_script("# N\nFirst").unwrap();

    app.handle_input_event(InputEvent::PointerDown {
        position: Vec2::new(800.0, 252.0 + 2.0 * 58.0 + 10.0),
    });
    app.tick(0);

    assert_eq!(app.title_screen_mode, TitleScreenMode::Load);
}

#[test]
fn title_screen_mouse_hover_selects_menu_item_without_activation() {
    let mut app = SuzuApp::new(title_config());
    app.load_script("# N\nFirst").unwrap();

    app.handle_input_event(InputEvent::PointerMove {
        position: Vec2::new(800.0, 252.0 + 4.0 * 58.0 + 10.0),
    });
    app.tick(0);

    assert_eq!(app.selected_title_menu_action(), TitleMenuAction::Quit);
    assert!(!app.quit_requested());
    assert!(app.title_screen_visible());
}

fn title_config() -> GameConfig {
    GameConfig {
        title_screen: crate::config::TitleScreenConfig {
            enabled: true,
            title: "Title".to_owned(),
            subtitle: "Subtitle".to_owned(),
            ..crate::config::TitleScreenConfig::default()
        },
        ..GameConfig::default()
    }
}
