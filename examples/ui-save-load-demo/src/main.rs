use anyhow::Result;
use suzu_app::{GameConfig, SuzuApp};
use suzu_platform::{run_desktop, FrameTexture, WindowConfig};
use suzu_save::SaveThumbnail;

fn main() -> Result<()> {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(include_str!("../script/main.szs"))?;
    app.advance_until_waiting();
    register_fallback_textures(&mut app);
    seed_demo_save(&mut app);

    run_desktop(WindowConfig::default(), app)
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
