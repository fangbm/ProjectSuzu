use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{bail, Context, Result};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PackageManifest {
    format_version: u32,
    generated_at_unix_ms: u64,
    root: String,
    assets: Vec<AssetEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AssetEntry {
    id: String,
    path: String,
    kind: AssetKind,
    bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    checksum: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    packed_offset: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    packed_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "is_stored")]
    compression: AssetCompression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "snake_case")]
enum AssetCompression {
    #[default]
    Stored,
    Rle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum AssetKind {
    Texture,
    Audio,
    Script,
    Font,
    Video,
    Data,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PackerArgs {
    root: PathBuf,
    output: Option<PathBuf>,
    pack: Option<PathBuf>,
}

fn main() -> Result<()> {
    if env::args_os().len() == 1 {
        print_usage();
        pause_for_double_click();
        return Ok(());
    }

    let args = parse_args(env::args_os().skip(1))?;
    let manifest = build_manifest(&args.root)
        .with_context(|| format!("failed to build manifest for {}", args.root.display()))?;
    let json = serde_json::to_string_pretty(&manifest)?;

    if let Some(pack) = args.pack {
        write_archive(&args.root, manifest.clone(), &pack)
            .with_context(|| format!("failed to write archive {}", pack.display()))?;
    }

    if let Some(output) = args.output {
        if let Some(parent) = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&output, json)
            .with_context(|| format!("failed to write {}", output.display()))?;
    } else {
        println!("{json}");
    }

    Ok(())
}

fn print_usage() {
    println!("usage: suzu-packer <asset-root> [--output manifest.json] [--pack assets.suzupack]");
    println!();
    println!("examples:");
    println!("  suzu-packer examples\\hello-world --output target\\hello-world-assets.json");
    println!("  suzu-packer examples\\hello-world --pack target\\hello-world.suzupack");
}

fn pause_for_double_click() {
    #[cfg(windows)]
    {
        println!();
        println!("Press Enter to close...");
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
    }
}

fn parse_args<I>(args: I) -> Result<PackerArgs>
where
    I: IntoIterator,
    I::Item: Into<std::ffi::OsString>,
{
    let mut root = None;
    let mut output = None;
    let mut pack = None;
    let mut args = args.into_iter().map(Into::into).peekable();

    while let Some(arg) = args.next() {
        if arg == "--output" || arg == "-o" {
            let Some(value) = args.next() else {
                bail!("usage: suzu-packer <asset-root> [--output manifest.json] [--pack assets.suzupack]");
            };
            output = Some(PathBuf::from(value));
            continue;
        }
        if arg == "--pack" {
            let Some(value) = args.next() else {
                bail!("usage: suzu-packer <asset-root> [--output manifest.json] [--pack assets.suzupack]");
            };
            pack = Some(PathBuf::from(value));
            continue;
        }

        if root.is_some() {
            bail!(
                "usage: suzu-packer <asset-root> [--output manifest.json] [--pack assets.suzupack]"
            );
        }
        root = Some(PathBuf::from(arg));
    }

    let Some(root) = root else {
        bail!("usage: suzu-packer <asset-root> [--output manifest.json] [--pack assets.suzupack]");
    };

    Ok(PackerArgs { root, output, pack })
}

fn build_manifest(root: &Path) -> Result<PackageManifest> {
    let root = root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", root.display()))?;
    if !root.is_dir() {
        bail!("asset root must be a directory: {}", root.display());
    }

    let mut assets = Vec::new();
    collect_assets(&root, &root, &mut assets)?;
    assets.sort_by(|left, right| left.path.cmp(&right.path));

    Ok(PackageManifest {
        format_version: 1,
        generated_at_unix_ms: unix_time_ms(),
        root: normalize_path(&root),
        assets,
    })
}

fn collect_assets(root: &Path, current: &Path, assets: &mut Vec<AssetEntry>) -> Result<()> {
    for entry in fs::read_dir(current)
        .with_context(|| format!("failed to read directory {}", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            collect_assets(root, &path, assets)?;
            continue;
        }
        if !metadata.is_file() {
            continue;
        }

        let relative = path
            .strip_prefix(root)
            .with_context(|| format!("failed to relativize {}", path.display()))?;
        let relative_path = normalize_path(relative);
        assets.push(AssetEntry {
            id: asset_id(relative),
            kind: asset_kind(&path),
            path: relative_path,
            bytes: metadata.len(),
            checksum: None,
            packed_offset: None,
            packed_bytes: None,
            compression: AssetCompression::Stored,
        });
    }

    Ok(())
}

fn write_archive(root: &Path, mut manifest: PackageManifest, output: &Path) -> Result<()> {
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let root = root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", root.display()))?;
    manifest.root = String::new();

    let mut data_section = Vec::new();
    for entry in &mut manifest.assets {
        let asset_path = root.join(entry.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        let raw = fs::read(&asset_path)
            .with_context(|| format!("failed to read asset {}", asset_path.display()))?;
        let rle = rle_encode(&raw);
        let (packed, compression) = if !raw.is_empty() && rle.len() < raw.len() {
            (rle, AssetCompression::Rle)
        } else {
            (raw.clone(), AssetCompression::Stored)
        };

        entry.bytes = raw.len() as u64;
        entry.checksum = Some(checksum64(&raw));
        entry.packed_offset = Some(data_section.len() as u64);
        entry.packed_bytes = Some(packed.len() as u64);
        entry.compression = compression;
        data_section.extend_from_slice(&packed);
    }

    let manifest_json = serde_json::to_vec(&manifest)?;
    let mut archive = b"SUZUPACK1".to_vec();
    archive.extend_from_slice(&(manifest_json.len() as u64).to_le_bytes());
    archive.extend_from_slice(&manifest_json);
    archive.extend_from_slice(&data_section);
    fs::write(output, archive).with_context(|| format!("failed to write {}", output.display()))
}

fn rle_encode(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut encoded = Vec::new();
    let mut index = 0;
    while index < data.len() {
        let value = data[index];
        let mut count = 1_u8;
        while index + (count as usize) < data.len()
            && data[index + count as usize] == value
            && count < u8::MAX
        {
            count += 1;
        }
        encoded.push(count);
        encoded.push(value);
        index += count as usize;
    }
    encoded
}

fn checksum64(data: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn is_stored(compression: &AssetCompression) -> bool {
    *compression == AssetCompression::Stored
}

fn asset_id(relative: &Path) -> String {
    let without_extension = relative.with_extension("");
    normalize_path(&without_extension)
}

fn asset_kind(path: &Path) -> AssetKind {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png" | "jpg" | "jpeg" | "webp" | "avif") => AssetKind::Texture,
        Some("ogg" | "wav" | "flac" | "mp3") => AssetKind::Audio,
        Some("szs" | "lua") => AssetKind::Script,
        Some("ttf" | "otf" | "ttc") => AssetKind::Font,
        Some("mp4" | "webm" | "mkv") => AssetKind::Video,
        Some("json" | "ron" | "toml" | "yaml" | "yml") => AssetKind::Data,
        _ => AssetKind::Unknown,
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace("\\\\?\\", "")
        .replace('\\', "/")
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_root_and_output_args() {
        let args = parse_args(["assets", "--output", "manifest.json"]).unwrap();

        assert_eq!(args.root, PathBuf::from("assets"));
        assert_eq!(args.output, Some(PathBuf::from("manifest.json")));
        assert_eq!(args.pack, None);
    }

    #[test]
    fn builds_sorted_manifest_with_asset_kinds() {
        let root = test_dir("suzu-packer-manifest");
        fs::create_dir_all(root.join("bg")).unwrap();
        fs::create_dir_all(root.join("script")).unwrap();
        fs::write(root.join("bg").join("school.png"), [1_u8, 2, 3]).unwrap();
        fs::write(root.join("script").join("main.szs"), b"# N\nHi").unwrap();

        let manifest = build_manifest(&root).unwrap();

        assert_eq!(manifest.format_version, 1);
        assert_eq!(manifest.assets.len(), 2);
        assert_eq!(manifest.assets[0].id, "bg/school");
        assert_eq!(manifest.assets[0].kind, AssetKind::Texture);
        assert_eq!(manifest.assets[0].path, "bg/school.png");
        assert_eq!(manifest.assets[0].bytes, 3);
        assert_eq!(manifest.assets[1].id, "script/main");
        assert_eq!(manifest.assets[1].kind, AssetKind::Script);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn classifies_unknown_extensions() {
        assert_eq!(asset_kind(Path::new("notes.bin")), AssetKind::Unknown);
        assert_eq!(asset_id(Path::new("voice/eileen.ogg")), "voice/eileen");
    }

    #[test]
    fn writes_archive_with_packed_offsets_and_checksums() {
        let root = test_dir("suzu-packer-archive");
        fs::create_dir_all(root.join("data")).unwrap();
        fs::write(root.join("data").join("repeat.bin"), vec![7_u8; 16]).unwrap();
        let archive_path = root.join("assets.suzupack");
        let manifest = build_manifest(&root).unwrap();

        write_archive(&root, manifest, &archive_path).unwrap();

        let bytes = fs::read(&archive_path).unwrap();
        assert!(bytes.starts_with(b"SUZUPACK1"));
        let manifest_len_start = b"SUZUPACK1".len();
        let manifest_len_end = manifest_len_start + 8;
        let manifest_len = u64::from_le_bytes(
            bytes[manifest_len_start..manifest_len_end]
                .try_into()
                .unwrap(),
        ) as usize;
        let manifest_json = &bytes[manifest_len_end..manifest_len_end + manifest_len];
        let manifest = serde_json::from_slice::<serde_json::Value>(manifest_json).unwrap();
        let asset = &manifest["assets"][0];
        assert_eq!(asset["compression"], "rle");
        assert!(asset["checksum"].as_u64().is_some());
        assert_eq!(asset["packed_offset"], 0);

        let _ = fs::remove_dir_all(root);
    }

    fn test_dir(name: &str) -> PathBuf {
        let mut root = env::temp_dir();
        root.push(format!("{name}-{}", unix_time_ms()));
        let _ = fs::remove_dir_all(&root);
        root
    }
}
