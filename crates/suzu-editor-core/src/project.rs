use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectIndex {
    pub root: PathBuf,
    pub scripts: Vec<PathBuf>,
    pub resources: Vec<ProjectResource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectResource {
    pub path: PathBuf,
    pub id: String,
    pub kind: ResourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Image,
    Audio,
    Script,
    Other,
}

impl ProjectIndex {
    pub fn scan(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let mut scripts = Vec::new();
        let mut resources = Vec::new();

        scan_dir(&root, &root, &mut scripts, &mut resources)
            .with_context(|| format!("failed to scan {}", root.display()))?;
        scripts.sort();
        resources.sort_by(|left, right| left.path.cmp(&right.path));

        Ok(Self {
            root,
            scripts,
            resources,
        })
    }
}

fn scan_dir(
    root: &Path,
    dir: &Path,
    scripts: &mut Vec<PathBuf>,
    resources: &mut Vec<ProjectResource>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if matches!(name, "target" | ".git") {
                continue;
            }
            scan_dir(root, &path, scripts, resources)?;
            continue;
        }

        let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        let kind = resource_kind(&path);
        if kind == ResourceKind::Script {
            scripts.push(relative.clone());
        }
        resources.push(ProjectResource {
            id: path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or_default()
                .to_owned(),
            path: relative,
            kind,
        });
    }

    Ok(())
}

fn resource_kind(path: &Path) -> ResourceKind {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png" | "jpg" | "jpeg" | "webp") => ResourceKind::Image,
        Some("ogg" | "wav" | "mp3" | "flac") => ResourceKind::Audio,
        Some("szs") => ResourceKind::Script,
        _ => ResourceKind::Other,
    }
}
