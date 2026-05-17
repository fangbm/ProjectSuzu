use std::path::PathBuf;

use anyhow::Result;
use suzu_app::{GameConfig, SuzuApp, TitleScreenConfig};
use suzu_platform::{run_desktop, FrameTexture, WindowConfig};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut app = SuzuApp::new(template_config());
    app.register_textures_from_dir(root.join("assets"))?;
    app.load_script(include_str!("../script/main.szs"))?;
    if app.scene_textures.is_empty() {
        register_fallback_textures(&mut app);
    }

    run_desktop(WindowConfig::default(), app)
}

fn template_config() -> GameConfig {
    GameConfig {
        title_screen: TitleScreenConfig {
            enabled: true,
            title: "Project Suzu".to_owned(),
            subtitle: "Minimal VN Template".to_owned(),
            background_texture: Some("bg_room".to_owned()),
            ..TitleScreenConfig::default()
        },
        ..GameConfig::default()
    }
}

fn register_fallback_textures(app: &mut SuzuApp) {
    app.scene_textures.push(FrameTexture::new(
        "bg_room",
        2,
        2,
        vec![
            40, 50, 68, 255, 58, 70, 92, 255, 26, 34, 48, 255, 84, 96, 118, 255,
        ],
    ));
    app.scene_textures.push(FrameTexture::new(
        "hero",
        2,
        2,
        vec![
            220, 178, 190, 255, 242, 210, 216, 255, 176, 116, 146, 255, 232, 190, 204, 255,
        ],
    ));
}
