use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::Xp3Archive;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KrkrCompatibilityReport {
    pub archives: Vec<KrkrArchiveReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KrkrArchiveReport {
    pub path: PathBuf,
    pub entries: usize,
    pub script_entries: usize,
    pub protected_script_entries: usize,
    pub entrypoint_candidates: Vec<String>,
    pub parse_error: Option<String>,
}

impl KrkrCompatibilityReport {
    pub fn protected_script_entries(&self) -> usize {
        self.archives
            .iter()
            .map(|archive| archive.protected_script_entries)
            .sum()
    }

    pub fn script_entries(&self) -> usize {
        self.archives
            .iter()
            .map(|archive| archive.script_entries)
            .sum()
    }

    pub fn has_protected_entries(&self) -> bool {
        self.protected_script_entries() > 0
    }
}

pub fn probe_krkr_directory(root: impl AsRef<Path>) -> Result<KrkrCompatibilityReport> {
    let root = root.as_ref();
    let archives = scan_krkr_archives(root)?;

    Ok(KrkrCompatibilityReport { archives })
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
                    protected_script_entries: 0,
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
        let protected_script_entries = script_entries
            .iter()
            .filter(|entry| entry.protected)
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
            protected_script_entries,
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

#[cfg(test)]
mod tests {
    use super::*;

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
