use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KrkrCompatibilityReport {
    pub packinone: Option<PackinOneReport>,
    pub lose_emote_psb: Option<LoseEmotePsbReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackinOneReport {
    pub dll_path: PathBuf,
    pub uses_chacha_filter: bool,
    pub exposes_load_data_pack: bool,
    pub exposes_packinone_list: bool,
    pub exposes_cryptmode: bool,
    pub exposes_outeriv: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoseEmotePsbReport {
    pub dll_path: PathBuf,
    pub randomizer_seed: u32,
}

pub fn probe_krkr_directory(root: impl AsRef<Path>) -> Result<KrkrCompatibilityReport> {
    let root = root.as_ref();
    let plugin_dir = root.join("plugin");
    let packinone_path = plugin_dir.join("PackinOne.dll");
    let emote_driver_path = root.join("emotedriver.dll");

    let packinone = if packinone_path.exists() {
        let bytes = fs::read(&packinone_path)
            .with_context(|| format!("failed to read {}", packinone_path.display()))?;
        Some(PackinOneReport {
            dll_path: packinone_path,
            uses_chacha_filter: contains_ascii_or_utf16le(&bytes, "ChaCha")
                || contains_ascii_or_utf16le(&bytes, "BasicCryptFilter"),
            exposes_load_data_pack: contains_ascii_or_utf16le(&bytes, "loadDataPack"),
            exposes_packinone_list: contains_ascii_or_utf16le(&bytes, "PackinOneList"),
            exposes_cryptmode: contains_ascii_or_utf16le(&bytes, "cryptmode"),
            exposes_outeriv: contains_ascii_or_utf16le(&bytes, "outeriv"),
        })
    } else {
        None
    };

    let lose_emote_psb = if emote_driver_path.exists() {
        let bytes = fs::read(&emote_driver_path)
            .with_context(|| format!("failed to read {}", emote_driver_path.display()))?;
        if contains_ascii_or_utf16le(&bytes, "#cryptkey#")
            || contains_ascii_or_utf16le(&bytes, "391022973")
        {
            Some(LoseEmotePsbReport {
                dll_path: emote_driver_path,
                randomizer_seed: 391_022_973,
            })
        } else {
            None
        }
    } else {
        None
    };

    Ok(KrkrCompatibilityReport {
        packinone,
        lose_emote_psb,
    })
}

fn contains_ascii_or_utf16le(bytes: &[u8], needle: &str) -> bool {
    let utf16le = utf16le_bytes(needle);
    bytes
        .windows(needle.len())
        .any(|window| window == needle.as_bytes())
        || bytes
            .windows(utf16le.len())
            .any(|window| window == utf16le.as_slice())
}

fn utf16le_bytes(value: &str) -> Vec<u8> {
    value
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_ascii_and_utf16le_needles() {
        assert!(contains_ascii_or_utf16le(b"before ChaCha after", "ChaCha"));
        assert!(contains_ascii_or_utf16le(
            &utf16le_bytes("loadDataPack"),
            "loadDataPack"
        ));
        assert!(!contains_ascii_or_utf16le(b"plain", "PackinOne"));
    }
}
