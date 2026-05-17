#![cfg_attr(windows, windows_subsystem = "windows")]

mod error_dialog;

use anyhow::Result;
use suzu_app::{GameConfig, SuzuApp, TitleScreenConfig};
use suzu_platform::{run_desktop, FrameTexture, WindowConfig};
use suzu_save::SaveThumbnail;

fn main() {
    if let Err(error) = run() {
        error_dialog::report_startup_error(&error);
    }
}

fn run() -> Result<()> {
    let mut app = SuzuApp::new(example_config());
    app.load_script(include_str!("../script/main.szs"))?;
    register_fallback_textures(&mut app);
    app.start_game();
    seed_demo_save(&mut app);
    app.show_title_screen();

    run_desktop(WindowConfig::default(), app)
}

fn example_config() -> GameConfig {
    GameConfig {
        title_screen: TitleScreenConfig {
            enabled: true,
            title: "Project Suzu".to_owned(),
            subtitle: "Save Load Demo".to_owned(),
            background_texture: Some("bg_menu_room".to_owned()),
            ..TitleScreenConfig::default()
        },
        ..GameConfig::default()
    }
}

fn seed_demo_save(app: &mut SuzuApp) {
    let thumbnail = SaveThumbnail::new(
        2,
        2,
        vec![
            32, 40, 64, 255, 72, 88, 128, 255, 48, 56, 84, 255, 120, 96, 132, 255,
        ],
    )
    .expect("thumbnail dimensions match rgba data");
    let _ = app.save_slot_with_thumbnail(0, thumbnail);
}

fn register_fallback_textures(app: &mut SuzuApp) {
    app.scene_textures.push(FrameTexture::new(
        "bg_menu_room",
        2,
        2,
        vec![
            24, 30, 44, 255, 38, 48, 70, 255, 18, 24, 36, 255, 58, 68, 96, 255,
        ],
    ));
}
