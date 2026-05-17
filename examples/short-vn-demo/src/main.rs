#![cfg_attr(windows, windows_subsystem = "windows")]

use std::path::PathBuf;

use anyhow::Result;
use suzu_app::{GameConfig, SuzuApp, TitleScreenConfig};
use suzu_platform::{run_desktop, FrameTexture, WindowConfig};
use suzu_save::SaveThumbnail;

mod error_dialog;

fn main() {
    if let Err(error) = run() {
        error_dialog::report_startup_error(&error);
    }
}

fn run() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut app = SuzuApp::new(demo_config());
    app.register_textures_from_dir(root.join("assets"))?;
    app.load_script(include_str!("../script/main.szs"))?;
    if app.scene_textures.is_empty() {
        register_fallback_textures(&mut app);
    }

    app.start_game();
    app.advance_until_waiting();
    seed_demo_save(&mut app);
    app.show_title_screen();

    run_desktop(WindowConfig::default(), app)
}

fn demo_config() -> GameConfig {
    GameConfig {
        title_screen: TitleScreenConfig {
            enabled: true,
            title: "Project Suzu".to_owned(),
            subtitle: "Short VN Demo".to_owned(),
        },
        ..GameConfig::default()
    }
}

fn seed_demo_save(app: &mut SuzuApp) {
    let thumbnail = SaveThumbnail::new(
        2,
        2,
        vec![
            40, 54, 84, 255, 96, 134, 170, 255, 36, 40, 62, 255, 196, 154, 118, 255,
        ],
    )
    .expect("thumbnail dimensions match rgba data");
    let _ = app.save_slot_with_thumbnail(0, thumbnail);
}

fn register_fallback_textures(app: &mut SuzuApp) {
    for (id, rgba) in [
        (
            "bg_station_morning",
            [
                64, 92, 128, 255, 122, 160, 184, 255, 238, 196, 128, 255, 48, 62, 90, 255,
            ],
        ),
        (
            "bg_library_afternoon",
            [
                92, 64, 58, 255, 144, 106, 78, 255, 52, 62, 66, 255, 210, 170, 112, 255,
            ],
        ),
        (
            "bg_platform_evening",
            [
                52, 54, 82, 255, 112, 78, 120, 255, 214, 136, 92, 255, 28, 34, 58, 255,
            ],
        ),
        (
            "suzu_smile",
            [
                236, 184, 196, 255, 248, 216, 222, 255, 184, 112, 150, 255, 222, 160, 184, 255,
            ],
        ),
        (
            "suzu_thinking",
            [
                202, 178, 220, 255, 226, 214, 240, 255, 142, 112, 174, 255, 190, 154, 214, 255,
            ],
        ),
        (
            "ren",
            [
                164, 192, 204, 255, 210, 228, 232, 255, 92, 126, 148, 255, 142, 174, 190, 255,
            ],
        ),
    ] {
        app.scene_textures
            .push(FrameTexture::new(id, 2, 2, rgba.to_vec()));
    }
}
