use super::super::*;

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
