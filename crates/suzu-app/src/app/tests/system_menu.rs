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

#[test]
fn system_menu_pointer_click_activates_hit_item_only() {
    let mut app = SuzuApp::new(GameConfig::default());

    app.open_system_menu();
    app.handle_input_event(InputEvent::PointerDown {
        position: Vec2::new(500.0, 184.0 + 3.0 * 58.0 + 12.0),
    });
    app.tick(0);

    assert!(!app.system_menu_visible());
    assert!(app.history_visible());
}

#[test]
fn system_menu_pointer_hover_selects_without_activation() {
    let mut app = SuzuApp::new(GameConfig::default());

    app.open_system_menu();
    app.handle_input_event(InputEvent::PointerMove {
        position: Vec2::new(500.0, 184.0 + 5.0 * 58.0 + 12.0),
    });
    app.tick(0);

    assert_eq!(app.selected_system_menu_action(), SystemMenuAction::Quit);
    assert!(app.system_menu_visible());
    assert!(!app.quit_requested());
}

#[test]
fn system_menu_pointer_click_outside_does_not_confirm_selection() {
    let mut app = SuzuApp::new(GameConfig::default());

    app.open_system_menu();
    app.move_system_menu_selection(5);
    assert_eq!(app.selected_system_menu_action(), SystemMenuAction::Quit);

    app.handle_input_event(InputEvent::PointerDown {
        position: Vec2::new(16.0, 16.0),
    });
    app.tick(0);

    assert!(app.system_menu_visible());
    assert!(!app.quit_requested());
}
