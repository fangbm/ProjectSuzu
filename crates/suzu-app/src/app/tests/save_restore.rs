use super::super::*;

#[test]
fn save_state_preserves_message_box_visibility() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("# N\nLine\n@hidemsg").unwrap();

    app.advance_until_waiting();
    app.reveal_dialogue_now();
    assert!(app.advance_script());

    let state = app.capture_state();
    assert!(!state.scene.message_box_visible);

    let mut restored = SuzuApp::new(GameConfig::default());
    restored.restore_state(state);
    assert!(!restored.scene.message_box_visible);
    assert_eq!(restored.scene.dialogue.as_ref().unwrap().raw, "N: Line");
}

#[test]
fn save_slot_can_store_thumbnail() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("# N\nFirst").unwrap();
    app.advance_until_waiting();

    let thumbnail = SaveThumbnail::new(1, 1, vec![1, 2, 3, 255]).unwrap();
    assert!(app.save_slot_with_thumbnail(0, thumbnail));

    let saved = app.saves.load_slot(0).unwrap();
    let thumbnail = saved.metadata.thumbnail.as_ref().unwrap();
    assert_eq!(thumbnail.width, 1);
    assert_eq!(thumbnail.height, 1);
    assert_eq!(thumbnail.rgba, [1, 2, 3, 255]);
}

#[test]
fn save_state_preserves_call_stack() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@call goto=common\n# N\n主线\n*common\n# N\n共通\n@return")
        .unwrap();

    assert!(app.advance_script());
    let state = app.capture_state();

    assert_eq!(state.script.call_stack.len(), 1);
    let mut restored = SuzuApp::new(GameConfig::default());
    restored.restore_state(state);
    restored.advance_until_waiting();
    assert_eq!(restored.scene.dialogue.as_ref().unwrap().raw, "N: 共通");
}

#[test]
fn autosave_command_captures_scene_snapshot() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@savename text=\"第一章\"\n@bg file=\"school\"\n@autosave")
        .unwrap();

    app.advance_until_waiting();
    let autosave = app.saves.autosave().unwrap();

    assert_eq!(autosave.metadata.title, "第一章");
    assert_eq!(
        autosave.scene.background.as_ref().unwrap().texture_id,
        "school"
    );
    assert_eq!(autosave.script.line_number, 3);
    assert_eq!(autosave.script.pending_commands.len(), 3);
}

#[test]
fn save_slot_restore_resumes_from_script_position() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("# 艾琳\n第一句\n# 艾琳\n第二句").unwrap();
    app.advance_until_waiting();
    app.reveal_dialogue_now();

    assert!(app.save_slot(0));
    app.confirm();
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 第二句");

    assert!(app.load_slot(0));
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 第一句");

    app.confirm();
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 第二句");
}
