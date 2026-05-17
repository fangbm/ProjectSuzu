use super::super::*;

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
