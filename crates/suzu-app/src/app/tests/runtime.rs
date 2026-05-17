use super::super::*;

#[test]
fn script_advance_updates_scene_and_audio() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@bg file=\"school\"\n@playbgm file=\"theme\" loop=true\n# 艾琳\n你好")
        .unwrap();

    assert!(app.advance_script());
    assert_eq!(app.scene.background.as_ref().unwrap().texture_id, "school");

    assert!(app.advance_script());
    assert!(matches!(
        app.audio.bgm.current,
        Some(AudioSource::File { ref path, looping: true }) if path == "theme"
    ));

    assert!(app.advance_script());
    assert!(app.scene.dialogue.as_ref().unwrap().raw.contains("艾琳"));
    assert_eq!(
        app.scene.dialogue.as_ref().unwrap().reveal.revealed_chars,
        0
    );
}

#[test]
fn registered_texture_is_loaded_when_script_references_it() {
    let mut path = std::env::temp_dir();
    path.push("suzu-app-bg-texture.png");
    let image = image::RgbaImage::from_raw(1, 1, vec![1, 2, 3, 255]).unwrap();
    image.save(&path).unwrap();

    let mut app = SuzuApp::new(GameConfig::default());
    app.register_texture("school", &path);
    app.load_script("@bg file=\"school\"").unwrap();

    assert!(app.advance_script());
    assert_eq!(app.scene_textures.len(), 1);
    assert_eq!(app.scene_textures[0].id, "school");
    assert_eq!(app.scene_textures[0].rgba, vec![1, 2, 3, 255]);

    let _ = std::fs::remove_file(path);
}

#[test]
fn wait_command_pauses_script_then_resumes() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@wait time=500\n# N\nAfter").unwrap();

    app.advance_until_waiting();
    assert_eq!(app.wait_timer_ms, Some(500));
    assert!(app.scene.dialogue.is_none());

    app.tick(499);
    assert_eq!(app.wait_timer_ms, Some(1));
    assert!(app.scene.dialogue.is_none());

    app.tick(1);
    assert!(app.wait_timer_ms.is_none());
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: After");
}

#[test]
fn confirm_does_not_skip_wait_command() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@wait time=500\n# N\nAfter").unwrap();

    app.advance_until_waiting();
    app.confirm();

    assert_eq!(app.wait_timer_ms, Some(500));
    assert!(app.scene.dialogue.is_none());
}

#[test]
fn confirm_after_last_dialogue_clears_message_box() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("# N\nLast").unwrap();

    app.advance_until_waiting();
    app.reveal_dialogue_now();
    assert!(app.scene.message_box_visible);

    app.confirm();

    assert_eq!(app.script.position(), app.script.len());
    assert!(app.scene.dialogue.is_none());
    assert!(!app.scene.message_box_visible);
}

#[test]
fn message_box_visibility_commands_affect_frame_output() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("# N\nLine\n@hidemsg\n@showmsg").unwrap();

    app.advance_until_waiting();
    app.reveal_dialogue_now();

    let frame = app.update(0);
    assert!(frame
        .sprites
        .iter()
        .any(|sprite| sprite.texture_id == "dialogue"));
    assert!(frame.texts.iter().any(|text| text.content == "N"));
    assert!(frame.texts.iter().any(|text| text.content == "Line"));

    assert!(app.advance_script());
    let frame = app.update(0);
    assert!(!frame
        .sprites
        .iter()
        .any(|sprite| sprite.texture_id == "dialogue"));
    assert!(!frame.texts.iter().any(|text| text.content == "N"));
    assert!(!frame.texts.iter().any(|text| text.content == "Line"));

    assert!(app.advance_script());
    let frame = app.update(0);
    assert!(frame
        .sprites
        .iter()
        .any(|sprite| sprite.texture_id == "dialogue"));
    assert!(frame.texts.iter().any(|text| text.content == "N"));
    assert!(frame.texts.iter().any(|text| text.content == "Line"));
}

#[test]
fn dialogue_reveals_over_time() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("# 艾琳\n你好世界").unwrap();

    assert!(app.advance_script());
    assert_eq!(app.scene.dialogue.as_ref().unwrap().visible_text(), "");

    app.tick(100);
    assert!(!app
        .scene
        .dialogue
        .as_ref()
        .unwrap()
        .visible_text()
        .is_empty());
    assert!(!app.scene.dialogue.as_ref().unwrap().reveal.is_complete());

    app.reveal_dialogue_now();
    assert!(app.scene.dialogue.as_ref().unwrap().reveal.is_complete());
    assert_eq!(
        app.scene.dialogue.as_ref().unwrap().visible_text(),
        app.scene.dialogue.as_ref().unwrap().raw
    );
}

#[test]
fn user_settings_apply_dialogue_reveal_speed() {
    let mut app = SuzuApp::new(GameConfig::default());
    let mut settings = UserSettings::default();
    settings.text.speed_chars_per_second = 120.0;
    app.apply_user_settings(settings);
    app.load_script("# 艾琳\n你好").unwrap();

    app.advance_until_waiting();

    assert_eq!(
        app.scene
            .dialogue
            .as_ref()
            .unwrap()
            .reveal
            .speed_chars_per_second,
        120.0
    );
}

#[test]
fn dialogue_control_tags_are_not_displayed() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("# 艾琳\n你好[l][r]下一行").unwrap();

    app.advance_until_waiting();
    app.reveal_dialogue_now();

    assert_eq!(
        app.scene.dialogue.as_ref().unwrap().raw,
        "艾琳: 你好\n下一行"
    );
    assert_eq!(app.history[0].text, "你好\n下一行");
}

#[test]
fn dialogue_wait_tag_pauses_until_confirmed() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("# 艾琳\n前半[l]后半\n# 艾琳\n下一句")
        .unwrap();

    app.advance_until_waiting();
    app.tick(1000);

    assert_eq!(
        app.scene.dialogue.as_ref().unwrap().visible_text(),
        "艾琳: 前半"
    );
    assert!(!app.scene.dialogue.as_ref().unwrap().reveal.is_complete());

    app.handle_input_event(InputEvent::Confirm);
    app.tick(0);
    assert_eq!(
        app.scene.dialogue.as_ref().unwrap().visible_text(),
        "艾琳: 前半"
    );

    app.tick(1000);
    assert_eq!(
        app.scene.dialogue.as_ref().unwrap().visible_text(),
        "艾琳: 前半后半"
    );

    app.handle_input_event(InputEvent::Confirm);
    app.tick(0);
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 下一句");
}

#[test]
fn frame_contains_dialogue_and_choice_texts() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("# 艾琳\n你好\n@choice \"A\" goto=a\n*a\n# 艾琳\nA")
        .unwrap();
    app.advance_until_waiting();
    app.reveal_dialogue_now();
    app.confirm();

    let frame = app.update(0);
    assert!(frame.texts.iter().any(|text| text.content == "A"));
}
