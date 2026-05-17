use super::super::*;

#[test]
fn completed_dialogue_is_marked_as_read() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("# N\nFirst").unwrap();

    app.advance_until_waiting();
    assert!(!app.is_current_dialogue_read());

    app.reveal_dialogue_now();
    assert!(app.is_current_dialogue_read());
}

#[test]
fn save_state_preserves_read_dialogue_keys() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("# N\nFirst").unwrap();
    app.advance_until_waiting();
    app.reveal_dialogue_now();

    let state = app.capture_state();
    assert_eq!(state.read_dialogue_keys.len(), 1);

    let mut restored = SuzuApp::new(GameConfig::default());
    restored.restore_state(state);
    assert_eq!(restored.read_dialogue_keys.len(), 1);
}

#[test]
fn history_overlay_scrolls_and_renders_entries() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("# N\nFirst\n# N\nSecond").unwrap();

    app.advance_until_waiting();
    app.reveal_dialogue_now();
    app.confirm();
    app.reveal_dialogue_now();
    app.open_history();

    assert!(app.history_visible());
    assert_eq!(app.visible_history_entries(1)[0].text, "Second");
    app.scroll_history(1);
    assert_eq!(app.visible_history_entries(1)[0].text, "First");

    let frame = app.update(0);
    assert!(frame
        .sprites
        .iter()
        .any(|sprite| sprite.texture_id == "history_backlog"));
    assert!(frame
        .texts
        .iter()
        .any(|text| text.content.contains("First")));
}

#[test]
fn history_voice_replay_uses_entry_voice_file() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@voice file=\"voices/line.ogg\"\n# N\nVoiced")
        .unwrap();

    app.advance_until_waiting();
    app.reveal_dialogue_now();
    app.open_history();

    assert!(app.replay_history_voice(0));
    assert!(matches!(
        app.audio.voice.current,
        Some(AudioSource::File { ref path, looping: false }) if path == "voices/line.ogg"
    ));
}

#[test]
fn cancel_input_closes_history_overlay() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.open_history();

    app.handle_input_event(InputEvent::Cancel);
    app.tick(0);

    assert!(!app.history_visible());
}
