#![cfg_attr(windows, windows_subsystem = "windows")]

mod error_dialog;

use anyhow::Result;
use suzu_app::{GameConfig, SuzuApp, TitleScreenConfig};
use suzu_platform::{run_desktop, FrameTexture, WindowConfig};

fn main() {
    if let Err(error) = run() {
        error_dialog::report_startup_error(&error);
    }
}

fn run() -> Result<()> {
    let mut app = SuzuApp::new(example_config());
    app.register_textures_from_dir("examples/hello-world/assets")?;
    app.load_script(include_str!("../script/main.szs"))?;
    if app.scene_textures.is_empty() {
        register_fallback_textures(&mut app);
    }

    run_desktop(WindowConfig::default(), app)
}

fn example_config() -> GameConfig {
    GameConfig {
        title_screen: TitleScreenConfig {
            enabled: true,
            title: "Project Suzu".to_owned(),
            subtitle: "Hello World".to_owned(),
            ..TitleScreenConfig::default()
        },
        ..GameConfig::default()
    }
}

fn register_fallback_textures(app: &mut SuzuApp) {
    app.scene_textures.push(FrameTexture::new(
        "bg_school_evening",
        2,
        2,
        vec![
            44, 55, 76, 255, 54, 69, 96, 255, 32, 42, 62, 255, 66, 82, 112, 255,
        ],
    ));
    app.scene_textures.push(FrameTexture::new(
        "bg_rooftop_evening",
        2,
        2,
        vec![
            78, 60, 94, 255, 104, 78, 120, 255, 42, 38, 68, 255, 138, 92, 116, 255,
        ],
    ));
    app.scene_textures.push(FrameTexture::new(
        "eileen",
        2,
        2,
        vec![
            220, 174, 188, 255, 240, 205, 212, 255, 184, 120, 148, 255, 232, 188, 202, 255,
        ],
    ));
}
