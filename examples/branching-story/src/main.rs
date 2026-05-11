use anyhow::Result;
use suzu_app::{GameConfig, SuzuApp};
use suzu_platform::{run_desktop, FrameTexture, WindowConfig};

fn main() -> Result<()> {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(include_str!("../script/main.szs"))?;
    app.advance_until_waiting();
    register_fallback_textures(&mut app);

    run_desktop(WindowConfig::default(), app)
}

fn register_fallback_textures(app: &mut SuzuApp) {
    app.scene_textures.push(FrameTexture::new(
        "bg_school_evening",
        2,
        2,
        vec![
            36, 45, 66, 255, 50, 64, 94, 255, 28, 36, 54, 255, 70, 86, 120, 255,
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
