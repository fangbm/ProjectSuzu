use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use suzu_app::{GameConfig, SuzuApp};
use suzu_asset::{AssetId, AssetManager};
use suzu_editor_core::convert_krkr_ks_to_szs;
use suzu_script::compile_script;

#[test]
fn example_szs_scripts_compile() -> Result<()> {
    let mut scripts = Vec::new();
    collect_szs_scripts(&repo_root().join("examples"), &mut scripts)?;
    assert!(!scripts.is_empty(), "expected example .szs scripts");

    for script in scripts {
        let source = fs::read_to_string(&script)
            .with_context(|| format!("failed to read {}", script.display()))?;
        compile_script(&source)
            .with_context(|| format!("failed to compile {}", script.display()))?;
    }
    Ok(())
}

#[test]
fn runtime_advances_to_first_waiting_point() -> Result<()> {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@bg file=\"school\"\n# Narrator\nHello from smoke.")?;

    assert!(app.advance_until_waiting());
    assert_eq!(
        app.scene
            .background
            .as_ref()
            .map(|layer| layer.texture_id.as_str()),
        Some("school")
    );
    assert_eq!(
        app.scene
            .dialogue
            .as_ref()
            .map(|dialogue| dialogue.raw.as_str()),
        Some("Narrator: Hello from smoke.")
    );
    Ok(())
}

#[test]
fn packer_output_loads_through_asset_manager() -> Result<()> {
    let root = test_dir("package-load")?;
    let asset_root = root.join("assets");
    fs::create_dir_all(asset_root.join("script"))?;
    fs::write(asset_root.join("script").join("main.szs"), "# N\nPacked")?;

    let manifest = suzu_packer::build_manifest(&asset_root)?;
    let archive_path = root.join("assets.suzupack");
    suzu_packer::write_archive(&asset_root, manifest, &archive_path)?;

    let mut assets = AssetManager::default();
    let registered = assets.register_package_file(&archive_path)?;
    assert_eq!(registered, 1);
    assert_eq!(
        assets.load_asset_bytes(AssetId::from("script/main"))?,
        b"# N\nPacked"
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn save_restore_preserves_script_position_and_scene() -> Result<()> {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@bg file=\"school\"\n# N\nFirst\n# N\nSecond")?;
    app.advance_until_waiting();

    let saved = app.capture_state();
    let saved_line = saved.script.line_number;
    let saved_scene = saved.scene.clone();

    app.reveal_dialogue_now();
    app.confirm();
    assert_eq!(
        app.scene
            .dialogue
            .as_ref()
            .map(|dialogue| dialogue.raw.as_str()),
        Some("N: Second")
    );

    app.restore_state(saved);
    assert_eq!(app.script.position(), saved_line);
    assert_eq!(app.scene.dialogue, saved_scene.dialogue);
    assert_eq!(app.scene.background, saved_scene.background);
    assert_eq!(
        app.scene
            .dialogue
            .as_ref()
            .map(|dialogue| dialogue.raw.as_str()),
        Some("N: First")
    );
    Ok(())
}

#[test]
fn plaintext_xp3_registers_and_reads_resources() -> Result<()> {
    let root = test_dir("plaintext-xp3")?;
    let archive_path = root.join("data.xp3");
    write_plain_xp3(&archive_path, "script/main.szs", b"# N\nFrom XP3")?;

    let mut assets = AssetManager::default();
    let registered = assets.register_xp3_file(&archive_path)?;
    assert_eq!(registered, 1);
    assert_eq!(assets.load_asset_bytes("main")?, b"# N\nFrom XP3");

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn kag_fragment_converts_to_compilable_szs() -> Result<()> {
    let converted = convert_krkr_ks_to_szs(
        r#"; smoke
*start|Start
[bg storage="school.png" time=100]
[playbgm storage="theme.ogg"]
# Alice
Hello[l][r]
[link target=*yes]Yes[endlink]
*yes
[voice storage="alice.ogg"]Good.
"#,
        Some("smoke.ks"),
    );

    assert!(converted.source.contains("@bg file=\"school.png\""));
    assert!(converted.source.contains("@choice \"Yes\" goto=\"yes\""));
    compile_script(&converted.source)?;
    Ok(())
}

fn collect_szs_scripts(root: &Path, scripts: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_szs_scripts(&path, scripts)?;
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("szs"))
        {
            scripts.push(path);
        }
    }
    scripts.sort();
    Ok(())
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn test_dir(name: &str) -> Result<PathBuf> {
    let mut root = std::env::temp_dir();
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    root.push(format!("suzu-smoke-{name}-{nanos}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;
    Ok(root)
}

fn write_plain_xp3(path: &Path, name: &str, data: &[u8]) -> Result<()> {
    let segment_offset = suzu_asset::xp3_magic().len() as u64 + 8;
    let index_offset = segment_offset + data.len() as u64;
    let index = build_xp3_index(name, data.len() as u64, segment_offset);

    let mut bytes = suzu_asset::xp3_magic().to_vec();
    bytes.extend_from_slice(&index_offset.to_le_bytes());
    bytes.extend_from_slice(data);
    bytes.push(0);
    bytes.extend_from_slice(&(index.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&index);
    fs::write(path, bytes)?;
    Ok(())
}

fn build_xp3_index(name: &str, size: u64, segment_offset: u64) -> Vec<u8> {
    let mut info = Vec::new();
    info.extend_from_slice(&0_u32.to_le_bytes());
    info.extend_from_slice(&size.to_le_bytes());
    info.extend_from_slice(&size.to_le_bytes());
    let name_utf16 = name.encode_utf16().collect::<Vec<_>>();
    info.extend_from_slice(&(name_utf16.len() as u16).to_le_bytes());
    for ch in name_utf16 {
        info.extend_from_slice(&ch.to_le_bytes());
    }

    let mut segm = Vec::new();
    segm.extend_from_slice(&0_u32.to_le_bytes());
    segm.extend_from_slice(&segment_offset.to_le_bytes());
    segm.extend_from_slice(&size.to_le_bytes());
    segm.extend_from_slice(&size.to_le_bytes());

    let mut file = Vec::new();
    push_xp3_chunk(&mut file, b"info", &info);
    push_xp3_chunk(&mut file, b"segm", &segm);

    let mut index = Vec::new();
    push_xp3_chunk(&mut index, b"File", &file);
    index
}

fn push_xp3_chunk(output: &mut Vec<u8>, tag: &[u8; 4], body: &[u8]) {
    output.extend_from_slice(tag);
    output.extend_from_slice(&(body.len() as u64).to_le_bytes());
    output.extend_from_slice(body);
}
