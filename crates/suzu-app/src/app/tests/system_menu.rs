use super::super::*;

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
