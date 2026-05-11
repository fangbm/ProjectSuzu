use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};

use crate::{AssetCompression, AssetManifestEntry, PackageManifest};

const MAGIC: &[u8] = b"SUZUPACK1";
const MANIFEST_LEN_BYTES: usize = 8;

#[derive(Debug, Clone)]
pub struct PackageArchive {
    path: PathBuf,
    manifest: PackageManifest,
    data_start: usize,
}

impl PackageArchive {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read package archive {}", path.display()))?;
        if bytes.len() < MAGIC.len() + MANIFEST_LEN_BYTES || &bytes[..MAGIC.len()] != MAGIC {
            bail!("invalid package archive header: {}", path.display());
        }

        let manifest_len_start = MAGIC.len();
        let manifest_len_end = manifest_len_start + MANIFEST_LEN_BYTES;
        let manifest_len = u64::from_le_bytes(
            bytes[manifest_len_start..manifest_len_end]
                .try_into()
                .expect("slice length is fixed"),
        ) as usize;
        let manifest_start = manifest_len_end;
        let manifest_end = manifest_start + manifest_len;
        if manifest_end > bytes.len() {
            bail!("package manifest length exceeds archive size");
        }

        let manifest =
            serde_json::from_slice::<PackageManifest>(&bytes[manifest_start..manifest_end])
                .context("failed to parse package manifest")?;
        if manifest.format_version != 1 {
            bail!(
                "unsupported package archive manifest version {}",
                manifest.format_version
            );
        }

        Ok(Self {
            path: path.to_owned(),
            manifest,
            data_start: manifest_end,
        })
    }

    pub fn manifest(&self) -> &PackageManifest {
        &self.manifest
    }

    pub fn read_asset(&self, id: &str) -> Result<Vec<u8>> {
        let entry = self
            .manifest
            .assets
            .iter()
            .find(|entry| entry.id == id)
            .with_context(|| format!("asset `{id}` is not in package"))?;
        self.read_entry(entry)
    }

    pub fn read_entry(&self, entry: &AssetManifestEntry) -> Result<Vec<u8>> {
        let offset = entry
            .packed_offset
            .with_context(|| format!("asset `{}` does not have a packed offset", entry.id))?
            as usize;
        let packed_bytes = entry
            .packed_bytes
            .with_context(|| format!("asset `{}` does not have a packed size", entry.id))?
            as usize;
        let archive = fs::read(&self.path)
            .with_context(|| format!("failed to read package archive {}", self.path.display()))?;
        let start = self.data_start + offset;
        let end = start + packed_bytes;
        if end > archive.len() {
            bail!("packed data for `{}` exceeds archive size", entry.id);
        }

        let packed = &archive[start..end];
        let data = match entry.compression {
            AssetCompression::Stored => packed.to_vec(),
            AssetCompression::Rle => rle_decode(packed)?,
        };

        if data.len() as u64 != entry.bytes {
            bail!("unpacked size mismatch for `{}`", entry.id);
        }
        if let Some(expected) = entry.checksum {
            let actual = checksum64(&data);
            if actual != expected {
                bail!("checksum mismatch for `{}`", entry.id);
            }
        }
        Ok(data)
    }
}

#[cfg(test)]
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

fn rle_decode(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() % 2 != 0 {
        bail!("invalid rle payload length");
    }

    let mut decoded = Vec::new();
    for chunk in data.chunks_exact(2) {
        decoded.extend(std::iter::repeat(chunk[1]).take(chunk[0] as usize));
    }
    Ok(decoded)
}

pub fn checksum64(data: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn archive_magic() -> &'static [u8] {
    MAGIC
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetManifestEntry, AssetType};

    #[test]
    fn archive_reads_stored_and_rle_assets() {
        let root = test_dir("suzu-archive-read");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("test.suzupack");

        let stored = b"hello".to_vec();
        let raw_rle = vec![7_u8; 8];
        let packed_rle = rle_encode(&raw_rle);
        let manifest = PackageManifest {
            format_version: 1,
            generated_at_unix_ms: 1,
            root: PathBuf::new(),
            assets: vec![
                AssetManifestEntry {
                    id: "stored".to_owned(),
                    path: "stored.bin".to_owned(),
                    kind: AssetType::Data,
                    bytes: stored.len() as u64,
                    checksum: Some(checksum64(&stored)),
                    packed_offset: Some(0),
                    packed_bytes: Some(stored.len() as u64),
                    compression: AssetCompression::Stored,
                },
                AssetManifestEntry {
                    id: "rle".to_owned(),
                    path: "rle.bin".to_owned(),
                    kind: AssetType::Data,
                    bytes: raw_rle.len() as u64,
                    checksum: Some(checksum64(&raw_rle)),
                    packed_offset: Some(stored.len() as u64),
                    packed_bytes: Some(packed_rle.len() as u64),
                    compression: AssetCompression::Rle,
                },
            ],
        };
        let manifest_json = serde_json::to_vec(&manifest).unwrap();
        let mut archive = MAGIC.to_vec();
        archive.extend_from_slice(&(manifest_json.len() as u64).to_le_bytes());
        archive.extend_from_slice(&manifest_json);
        archive.extend_from_slice(&stored);
        archive.extend_from_slice(&packed_rle);
        fs::write(&path, archive).unwrap();

        let archive = PackageArchive::from_file(&path).unwrap();
        assert_eq!(archive.read_asset("stored").unwrap(), stored);
        assert_eq!(archive.read_asset("rle").unwrap(), raw_rle);

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
