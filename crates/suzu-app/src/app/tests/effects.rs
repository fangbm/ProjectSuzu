use super::super::*;

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
