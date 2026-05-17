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
