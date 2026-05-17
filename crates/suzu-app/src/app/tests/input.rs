use super::super::*;

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
