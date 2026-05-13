use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::Xp3Archive;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KrkrCompatibilityReport {
    pub packinone: Option<PackinOneReport>,
    pub lose_emote_psb: Option<LoseEmotePsbReport>,
    pub archives: Vec<KrkrArchiveReport>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KrkrArchiveReport {
    pub path: PathBuf,
    pub entries: usize,
    pub script_entries: usize,
    pub encrypted_script_entries: usize,
    pub entrypoint_candidates: Vec<String>,
    pub parse_error: Option<String>,
}

impl KrkrCompatibilityReport {
    pub fn encrypted_script_entries(&self) -> usize {
        self.archives
            .iter()
            .map(|archive| archive.encrypted_script_entries)
            .sum()
    }

    pub fn script_entries(&self) -> usize {
        self.archives
            .iter()
            .map(|archive| archive.script_entries)
            .sum()
    }

    pub fn has_packinone_blocker(&self) -> bool {
        self.packinone.is_some() && self.encrypted_script_entries() > 0
    }
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

    let archives = scan_krkr_archives(root)?;

    Ok(KrkrCompatibilityReport {
        packinone,
        lose_emote_psb,
        archives,
    })
}

fn scan_krkr_archives(root: &Path) -> Result<Vec<KrkrArchiveReport>> {
    let mut archives = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("failed to scan {}", root.display()))? {
        let path = entry?.path();
        if !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("xp3"))
        {
            continue;
        }

        let archive = match Xp3Archive::from_file(&path) {
            Ok(archive) => archive,
            Err(error) => {
                archives.push(KrkrArchiveReport {
                    path,
                    entries: 0,
                    script_entries: 0,
                    encrypted_script_entries: 0,
                    entrypoint_candidates: Vec::new(),
                    parse_error: Some(format!("{error:#}")),
                });
                continue;
            }
        };

        let script_entries = archive
            .entries()
            .iter()
            .filter(|entry| krkr_script_like_entry(&entry.name))
            .collect::<Vec<_>>();
        let encrypted_script_entries = script_entries
            .iter()
            .filter(|entry| entry.encrypted)
            .count();
        let mut entrypoint_candidates = script_entries
            .iter()
            .filter(|entry| krkr_entry_looks_like_entrypoint(&entry.name))
            .map(|entry| entry.name.clone())
            .collect::<Vec<_>>();
        entrypoint_candidates.sort_by_key(|name| name.to_ascii_lowercase());
        entrypoint_candidates.truncate(16);

        archives.push(KrkrArchiveReport {
            path,
            entries: archive.entries().len(),
            script_entries: script_entries.len(),
            encrypted_script_entries,
            entrypoint_candidates,
            parse_error: None,
        });
    }
    archives.sort_by_key(|archive| {
        archive
            .path
            .file_name()
            .map(|name| name.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default()
    });
    Ok(archives)
}

fn krkr_script_like_entry(path: &str) -> bool {
    let Some(extension) = Path::new(path).extension().and_then(|value| value.to_str()) else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "ks" | "tjs" | "func"
    )
}

fn krkr_entry_looks_like_entrypoint(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "startup.tjs"
            | "system/startup.tjs"
            | "scenario/start.ks"
            | "scenario/first.ks"
            | "scenario/main.ks"
            | "main/start.ks"
            | "main/first.ks"
            | "main/default.tjs"
    ) || normalized.ends_with("/startup.tjs")
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

    #[test]
    fn recognizes_krkr_script_like_entries() {
        assert!(krkr_script_like_entry("main/default.tjs"));
        assert!(krkr_script_like_entry("main/first.ks"));
        assert!(krkr_script_like_entry("main/dialog.func"));
        assert!(!krkr_script_like_entry("main/cglist.csv"));
        assert!(!krkr_script_like_entry("image/bg.png"));
    }

    #[test]
    fn recognizes_common_entrypoints() {
        assert!(krkr_entry_looks_like_entrypoint("startup.tjs"));
        assert!(krkr_entry_looks_like_entrypoint("main/default.tjs"));
        assert!(krkr_entry_looks_like_entrypoint("scenario/first.ks"));
        assert!(!krkr_entry_looks_like_entrypoint("main/custom.ks"));
    }
}
