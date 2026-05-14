use super::*;

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
fn bgm_commands_apply_fade_timing() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@playbgm file=\"theme\" loop=true fadein=1000\n@stopbgm fadeout=1000")
        .unwrap();

    assert!(app.advance_script());
    assert_eq!(app.audio.bgm.volume, 0.0);

    app.tick(500);
    assert_eq!(app.audio.bgm.volume, 0.5);

    app.tick(500);
    assert_eq!(app.audio.bgm.volume, 1.0);
    assert!(app.audio.bgm.current.is_some());

    assert!(app.advance_script());
    app.tick(500);
    assert_eq!(app.audio.bgm.volume, 0.5);
    assert!(app.audio.bgm.current.is_some());

    app.tick(500);
    assert!(app.audio.bgm.current.is_none());
    assert_eq!(app.audio.bgm.volume, 1.0);
}

#[test]
fn user_settings_apply_audio_volumes() {
    let mut app = SuzuApp::new(GameConfig::default());
    let mut settings = UserSettings::default();
    settings.audio.master_volume = 0.8;
    settings.audio.bgm_volume = 0.6;
    settings.audio.voice_volume = 0.7;
    settings.audio.se_volume = 0.5;

    app.apply_user_settings(settings);

    assert_eq!(app.audio.master_volume, 0.8);
    assert_eq!(app.audio.bgm_volume, 0.6);
    assert_eq!(app.audio.voice_volume, 0.7);
    assert_eq!(app.audio.se_volume, 0.5);
}

#[test]
fn user_settings_clamp_audio_volumes() {
    let mut app = SuzuApp::new(GameConfig::default());
    let mut settings = UserSettings::default();
    settings.audio.master_volume = 2.0;
    settings.audio.bgm_volume = -1.0;

    app.apply_user_settings(settings);

    assert_eq!(app.audio.master_volume, 1.0);
    assert_eq!(app.audio.bgm_volume, 0.0);
}

#[test]
fn voice_commands_apply_to_voice_channel() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(
        "@playvoice file=\"voices/eileen_001.ogg\" fadein=1000\n@stopvoice fadeout=1000",
    )
    .unwrap();

    assert!(app.advance_script());
    assert_eq!(app.audio.voice.volume, 0.0);
    assert!(matches!(
        app.audio.voice.current,
        Some(AudioSource::File { ref path, looping: false }) if path == "voices/eileen_001.ogg"
    ));

    app.tick(500);
    assert_eq!(app.audio.voice.volume, 0.5);
    app.tick(500);
    assert_eq!(app.audio.voice.volume, 1.0);

    assert!(app.advance_script());
    app.tick(500);
    assert_eq!(app.audio.voice.volume, 0.5);
    assert!(app.audio.voice.current.is_some());

    app.tick(500);
    assert!(app.audio.voice.current.is_none());
    assert_eq!(app.audio.voice.volume, 1.0);
}

#[test]
fn voice_command_cues_next_dialogue_line() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@voice file=\"voices/eileen_001.ogg\" fadein=200\n# 艾琳\n你好")
        .unwrap();

    assert!(app.advance_script());
    assert!(app.audio.voice.current.is_none());
    assert_eq!(
        app.pending_voice.as_ref().unwrap().file,
        "voices/eileen_001.ogg"
    );

    assert!(app.advance_script());
    assert_eq!(app.audio.voice.volume, 0.0);
    assert!(matches!(
        app.audio.voice.current,
        Some(AudioSource::File { ref path, looping: false }) if path == "voices/eileen_001.ogg"
    ));
    assert!(app
        .scene
        .dialogue
        .as_ref()
        .unwrap()
        .segments
        .iter()
        .any(|segment| matches!(
            segment,
            TextSegment::VoiceSync { char_index: 0, voice_file } if voice_file == "voices/eileen_001.ogg"
        )));
    assert!(app.pending_voice.is_none());
}

#[test]
fn queued_voice_is_replaced_by_later_voice_command() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@voice file=\"voices/old.ogg\"\n@voice file=\"voices/new.ogg\"\n# 艾琳\n你好")
        .unwrap();

    assert!(app.advance_script());
    assert!(app.advance_script());
    assert_eq!(app.pending_voice.as_ref().unwrap().file, "voices/new.ogg");

    assert!(app.advance_script());
    assert!(matches!(
        app.audio.voice.current,
        Some(AudioSource::File { ref path, looping: false }) if path == "voices/new.ogg"
    ));
}

#[test]
fn save_state_preserves_voice_source() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@playvoice file=\"voices/eileen_001.ogg\"")
        .unwrap();

    assert!(app.advance_script());
    let state = app.capture_state();

    assert!(matches!(
        state.audio.voice,
        Some(AudioSource::File { ref path, looping: false }) if path == "voices/eileen_001.ogg"
    ));
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
fn animation_updates_sprite_state() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(
        "@char name=\"eileen\" pos=center\n@anim target=\"eileen\" type=zoom scale=1.25",
    )
    .unwrap();

    assert!(app.advance_script());
    assert!(app.advance_script());

    app.tick(500);

    assert_eq!(app.scene.characters[0].scale, Vec2::new(1.25, 1.25));
}

#[test]
fn character_face_selects_expression_texture_and_keeps_name_target() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(
        "@char name=\"eileen\" face=\"happy\" pos=center\n@anim target=\"eileen\" type=move_to x=520 y=20 duration=0",
    )
    .unwrap();

    assert!(app.advance_script());
    assert_eq!(app.scene.characters[0].entity_id.as_deref(), Some("eileen"));
    assert_eq!(app.scene.characters[0].texture_id, "eileen_happy");

    assert!(app.advance_script());
    assert_eq!(app.scene.characters[0].position, Vec2::new(520.0, 20.0));
}

#[test]
fn character_command_supports_custom_position() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@char name=\"eileen\" x=320 y=24 layer=4")
        .unwrap();

    assert!(app.advance_script());
    assert_eq!(app.scene.characters[0].position, Vec2::new(320.0, 24.0));
    assert_eq!(app.scene.characters[0].size, Vec2::new(360.0, 720.0));
    assert_eq!(app.scene.characters[0].z_index, 4);
}

#[test]
fn character_command_supports_custom_size() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@char name=\"eileen\" width=420 height=680")
        .unwrap();

    assert!(app.advance_script());
    assert_eq!(app.scene.characters[0].size, Vec2::new(420.0, 680.0));
}

#[test]
fn character_command_supports_horizontal_flip() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@char name=\"eileen\" flip=true").unwrap();

    assert!(app.advance_script());
    assert!(app.scene.characters[0].flip_x);

    let frame = app.update(0);
    let sprite = frame
        .sprites
        .iter()
        .find(|sprite| sprite.texture_id == "eileen")
        .unwrap();
    assert!(sprite.flip_x);
}

#[test]
fn fade_animation_updates_character_opacity_immediately() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(
        "@char name=\"eileen\" pos=center\n@anim target=\"eileen\" type=fade opacity=0.25",
    )
    .unwrap();

    assert!(app.advance_script());
    assert!(app.advance_script());

    assert_eq!(app.scene.characters[0].opacity, 0.25);
}

#[test]
fn repeated_character_command_updates_existing_layer() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(
        "@char name=\"eileen\" face=\"happy\" pos=center layer=10\n@anim target=\"eileen\" type=zoom scale=1.2\n@char name=\"eileen\" face=\"blush\" pos=right layer=12",
    )
    .unwrap();

    assert!(app.advance_script());
    assert!(app.advance_script());
    assert!(app.advance_script());

    assert_eq!(app.scene.characters.len(), 1);
    assert_eq!(app.scene.characters[0].texture_id, "eileen_blush");
    assert_eq!(app.scene.characters[0].position, Vec2::new(740.0, 0.0));
    assert_eq!(app.scene.characters[0].size, Vec2::new(360.0, 720.0));
    assert_eq!(app.scene.characters[0].scale, Vec2::new(1.2, 1.2));
    assert!(!app.scene.characters[0].flip_x);
    assert_eq!(app.scene.characters[0].z_index, 12);
}

#[test]
fn repeated_character_command_updates_existing_flip() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@char name=\"eileen\"\n@char name=\"eileen\" flip=true")
        .unwrap();

    assert!(app.advance_script());
    assert!(app.advance_script());

    assert_eq!(app.scene.characters.len(), 1);
    assert!(app.scene.characters[0].flip_x);
}

#[test]
fn repeated_character_command_updates_existing_size() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(
        "@char name=\"eileen\" width=360 height=720\n@char name=\"eileen\" width=420 height=680",
    )
    .unwrap();

    assert!(app.advance_script());
    assert!(app.advance_script());

    assert_eq!(app.scene.characters.len(), 1);
    assert_eq!(app.scene.characters[0].size, Vec2::new(420.0, 680.0));
}

#[test]
fn hide_character_removes_layer_and_pending_animation() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(
        "@char name=\"eileen\" pos=center\n@anim target=\"eileen\" type=move_to x=560 y=40 duration=1000\n@hidechar name=\"eileen\"",
    )
    .unwrap();

    assert!(app.advance_script());
    assert!(app.advance_script());
    assert_eq!(app.scene.characters.len(), 1);
    assert_eq!(app.active_animations.len(), 1);

    assert!(app.advance_script());
    assert!(app.scene.characters.is_empty());
    assert!(app.active_animations.is_empty());
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
fn animation_interpolates_over_time() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(
        "@char name=\"eileen\" pos=center\n@anim target=\"eileen\" type=move_to x=560 y=40 duration=1000",
    )
    .unwrap();

    assert!(app.advance_script());
    assert!(app.advance_script());

    app.tick(500);
    let halfway = app.scene.characters[0].position;
    assert!(halfway.x > 460.0 && halfway.x < 560.0);
    assert!(halfway.y > 0.0 && halfway.y < 40.0);

    app.tick(500);
    assert_eq!(app.scene.characters[0].position, Vec2::new(560.0, 40.0));
}

#[test]
fn fade_animation_interpolates_over_time() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(
        "@char name=\"eileen\" pos=center\n@anim target=\"eileen\" type=fade opacity=0 duration=1000",
    )
    .unwrap();

    assert!(app.advance_script());
    assert!(app.advance_script());

    app.tick(500);
    assert!(app.scene.characters[0].opacity > 0.0);
    assert!(app.scene.characters[0].opacity < 1.0);

    app.tick(500);
    assert_eq!(app.scene.characters[0].opacity, 0.0);
}

#[test]
fn background_crossfade_interpolates_opacity() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@bg file=\"school\"\n@bg file=\"rooftop\" time=1000 method=crossfade")
        .unwrap();

    assert!(app.advance_script());
    assert_eq!(app.scene.background.as_ref().unwrap().texture_id, "school");

    assert!(app.advance_script());
    assert_eq!(app.scene.background.as_ref().unwrap().texture_id, "rooftop");
    assert!(app.scene.outgoing_background.is_some());

    app.tick(500);
    let incoming = app.scene.background.as_ref().unwrap().opacity;
    let outgoing = app.scene.outgoing_background.as_ref().unwrap().opacity;
    assert!(incoming > 0.0 && incoming < 1.0);
    assert!(outgoing > 0.0 && outgoing < 1.0);

    app.tick(500);
    assert_eq!(app.scene.background.as_ref().unwrap().opacity, 1.0);
    assert!(app.scene.outgoing_background.is_none());
}

#[test]
fn background_fade_through_color_uses_overlay_and_delays_incoming_opacity() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(
        "@bg file=\"school\"\n@bg file=\"rooftop\" time=1000 method=fade_through_color color=#112233",
    )
    .unwrap();

    assert!(app.advance_script());
    assert!(app.advance_script());

    app.tick(250);
    assert_eq!(app.scene.background.as_ref().unwrap().opacity, 0.0);
    assert!(app.scene.outgoing_background.as_ref().unwrap().opacity < 1.0);

    let frame = app.update(0);
    let overlay = frame
        .sprites
        .iter()
        .find(|sprite| sprite.texture_id == "bg_transition_color")
        .unwrap();
    assert_eq!(
        overlay.tint,
        Color::rgba(
            0x11 as f32 / 255.0,
            0x22 as f32 / 255.0,
            0x33 as f32 / 255.0,
            1.0
        )
    );
    assert!(overlay.opacity > 0.0);

    app.tick(750);
    assert_eq!(app.scene.background.as_ref().unwrap().opacity, 1.0);
    assert!(app.scene.outgoing_background.is_none());
    let frame = app.update(0);
    assert!(!frame
        .sprites
        .iter()
        .any(|sprite| sprite.texture_id == "bg_transition_color"));
}

#[test]
fn flash_effect_adds_fading_overlay() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@fx type=flash color=#FF0000 duration=1000\n# N\nFlash")
        .unwrap();

    app.advance_until_waiting();
    let frame = app.update(0);
    let flash = frame
        .sprites
        .iter()
        .find(|sprite| sprite.texture_id == "fx_flash")
        .unwrap();
    assert_eq!(flash.tint, Color::rgba(1.0, 0.0, 0.0, 1.0));
    assert_eq!(flash.opacity, 1.0);

    let frame = app.update(500);
    let flash = frame
        .sprites
        .iter()
        .find(|sprite| sprite.texture_id == "fx_flash")
        .unwrap();
    assert!(flash.opacity > 0.0 && flash.opacity < 1.0);

    let frame = app.update(500);
    assert!(!frame
        .sprites
        .iter()
        .any(|sprite| sprite.texture_id == "fx_flash"));
}

#[test]
fn quake_effect_offsets_frame_layers_temporarily() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@bg file=\"school\"\n@fx type=quake intensity=12 duration=1000\n# N\nShake")
        .unwrap();

    app.advance_until_waiting();
    let frame = app.update(128);
    let background = frame
        .sprites
        .iter()
        .find(|sprite| sprite.texture_id == "school")
        .unwrap();
    assert_ne!(background.bounds.origin, Vec2::ZERO);

    let frame = app.update(1000);
    let background = frame
        .sprites
        .iter()
        .find(|sprite| sprite.texture_id == "school")
        .unwrap();
    assert_eq!(background.bounds.origin, Vec2::ZERO);
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
fn completed_dialogue_is_marked_as_read() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("# N\nFirst").unwrap();

    app.advance_until_waiting();
    assert!(!app.is_current_dialogue_read());

    app.reveal_dialogue_now();
    assert!(app.is_current_dialogue_read());
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

#[test]
fn conditional_choices_filter_by_variables() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(
        "@set name=affection_eileen value=52\n@choice \"普通路线\" goto=normal\n@choice \"艾琳路线\" goto=eileen cond=affection_eileen>=50\n*normal\n# N\n普通\n*eileen\n# N\n艾琳",
    )
    .unwrap();

    app.advance_until_waiting();
    assert_eq!(app.scene.choice.as_ref().unwrap().options.len(), 2);
    app.handle_input_event(InputEvent::MoveSelection { delta: 1 });
    app.handle_input_event(InputEvent::Confirm);
    app.tick(0);

    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: 艾琳");
}

#[test]
fn false_conditional_choice_does_not_block_script() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@set name=seen_secret value=false\n@choice \"秘密\" goto=secret cond=seen_secret\n# N\n继续\n*secret\n# N\n秘密")
        .unwrap();

    app.advance_until_waiting();

    assert!(app.scene.choice.is_none());
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: 继续");
}

#[test]
fn if_block_inserts_commands_when_condition_is_true() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@set name=affection_eileen value=52\n@if cond=affection_eileen>=50\n# 艾琳\n条件成立\n@endif\n# 艾琳\n结束")
        .unwrap();

    app.advance_until_waiting();

    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 条件成立");
    app.reveal_dialogue_now();
    app.confirm();
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 结束");
}

#[test]
fn if_block_skips_commands_when_condition_is_false() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@set name=affection_eileen value=10\n@if cond=affection_eileen>=50\n# 艾琳\n条件成立\n@endif\n# 艾琳\n结束")
        .unwrap();

    app.advance_until_waiting();

    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 结束");
}

#[test]
fn if_else_block_runs_else_when_condition_is_false() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@set name=affection_eileen value=10\n@if cond=affection_eileen>=50\n# 艾琳\n真\n@else\n# 艾琳\n假\n@endif")
        .unwrap();

    app.advance_until_waiting();

    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 假");
}

#[test]
fn call_and_return_resume_after_subroutine() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@call goto=common\n# N\n主线\n*common\n# N\n共通\n@return\n# N\n不会重复")
        .unwrap();

    app.advance_until_waiting();
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: 共通");

    app.reveal_dialogue_now();
    app.confirm();
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: 主线");
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
