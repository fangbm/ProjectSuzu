use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::AssetType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    pub format_version: u32,
    pub generated_at_unix_ms: u64,
    pub root: PathBuf,
    pub assets: Vec<AssetManifestEntry>,
}

impl PackageManifest {
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .with_context(|| format!("failed to read manifest {}", path.display()))?;
        let manifest = serde_json::from_str::<Self>(&source)
            .with_context(|| format!("failed to parse manifest {}", path.display()))?;
        if manifest.format_version != 1 {
            bail!(
                "unsupported asset manifest version {}",
                manifest.format_version
            );
        }
        Ok(manifest)
    }

    pub fn asset_path(&self, entry: &AssetManifestEntry) -> PathBuf {
        self.root
            .join(entry.path.replace('/', std::path::MAIN_SEPARATOR_STR))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManifestEntry {
    pub id: String,
    pub path: String,
    pub kind: AssetType,
    pub bytes: u64,
    #[serde(default)]
    pub checksum: Option<u64>,
    #[serde(default)]
    pub packed_offset: Option<u64>,
    #[serde(default)]
    pub packed_bytes: Option<u64>,
    #[serde(default)]
    pub compression: AssetCompression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssetCompression {
    #[default]
    Stored,
    Rle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_manifest_from_json_file() {
        let root = test_dir("suzu-asset-manifest");
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("manifest.json");
        fs::write(
            &manifest_path,
            r#"{
  "format_version": 1,
  "generated_at_unix_ms": 1,
  "root": "assets",
  "assets": [
    { "id": "bg/school", "path": "bg/school.png", "kind": "texture", "bytes": 3 }
  ]
}"#,
        )
        .unwrap();

        let manifest = PackageManifest::from_json_file(&manifest_path).unwrap();

        assert_eq!(manifest.root, PathBuf::from("assets"));
        assert_eq!(manifest.assets[0].kind, AssetType::Texture);
        assert_eq!(
            manifest.asset_path(&manifest.assets[0]),
            PathBuf::from("assets").join("bg").join("school.png")
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_unsupported_manifest_version() {
        let root = test_dir("suzu-asset-manifest-version");
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("manifest.json");
        fs::write(
            &manifest_path,
            r#"{
  "format_version": 99,
  "generated_at_unix_ms": 1,
  "root": "assets",
  "assets": []
}"#,
        )
        .unwrap();

        assert!(PackageManifest::from_json_file(&manifest_path).is_err());

        let _ = fs::remove_dir_all(root);
    }

    fn test_dir(name: &str) -> PathBuf {
        let mut root = std::env::temp_dir();
        root.push(format!(
            "{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }
}
