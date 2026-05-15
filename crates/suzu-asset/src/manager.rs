use std::collections::{HashMap, VecDeque};

use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use anyhow::{bail, Context, Result};

use crate::{
    AssetId, AssetType, PackageArchive, PackageManifest, TextureAsset, Xp3Archive, Xp3Options,
};

#[derive(Debug, Clone)]
pub struct AssetRecord {
    pub id: AssetId,
    pub asset_type: AssetType,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
struct Xp3AssetSource {
    archive_index: usize,
    entry_name: String,
}

#[derive(Debug, Clone)]
struct PackageAssetSource {
    archive_index: usize,
    asset_id: String,
}

#[derive(Debug, Default)]
pub struct AssetManager {
    records: HashMap<AssetId, AssetRecord>,
    package_archives: Vec<PackageArchive>,
    package_sources: HashMap<AssetId, PackageAssetSource>,
    xp3_archives: Vec<Xp3Archive>,
    xp3_sources: HashMap<AssetId, Xp3AssetSource>,
    texture_cache: HashMap<AssetId, TextureAsset>,
    texture_lru: VecDeque<AssetId>,
    texture_cache_capacity: Option<usize>,
}

pub struct AsyncTextureLoad {
    id: AssetId,
    handle: JoinHandle<Result<TextureAsset>>,
}

impl AsyncTextureLoad {
    pub fn id(&self) -> &AssetId {
        &self.id
    }

    pub fn wait(self) -> Result<TextureAsset> {
        self.handle
            .join()
            .map_err(|_| anyhow::anyhow!("texture loader thread panicked for `{}`", self.id.0))?
    }
}

impl AssetManager {
    pub fn register(&mut self, record: AssetRecord) {
        self.records.insert(record.id.clone(), record);
    }

    pub fn register_path(
        &mut self,
        id: impl Into<AssetId>,
        asset_type: AssetType,
        path: impl Into<PathBuf>,
    ) {
        self.register(AssetRecord {
            id: id.into(),
            asset_type,
            path: path.into(),
        });
    }

    pub fn register_texture(&mut self, id: impl Into<AssetId>, path: impl Into<PathBuf>) {
        self.register_path(id, AssetType::Texture, path);
    }

    pub fn get(&self, id: &AssetId) -> Option<&AssetRecord> {
        self.records.get(id)
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn set_texture_cache_capacity(&mut self, capacity: usize) {
        self.texture_cache_capacity = Some(capacity);
        self.evict_textures_to_capacity();
    }

    pub fn clear_texture_cache(&mut self) {
        self.texture_cache.clear();
        self.texture_lru.clear();
    }

    pub fn cached_texture_count(&self) -> usize {
        self.texture_cache.len()
    }

    pub fn register_manifest(&mut self, manifest: &PackageManifest) -> usize {
        let mut count = 0;
        for entry in &manifest.assets {
            if entry.kind == AssetType::Unknown {
                continue;
            }

            self.register_path(entry.id.clone(), entry.kind, manifest.asset_path(entry));
            count += 1;
        }
        count
    }

    pub fn register_manifest_file(&mut self, path: impl AsRef<Path>) -> Result<usize> {
        let manifest = PackageManifest::from_json_file(path)?;
        Ok(self.register_manifest(&manifest))
    }

    pub fn register_package_file(&mut self, path: impl AsRef<Path>) -> Result<usize> {
        let path = path.as_ref();
        let archive = PackageArchive::from_file(path)?;
        let archive_index = self.package_archives.len();
        let mut count = 0;

        for entry in &archive.manifest().assets {
            if entry.kind == AssetType::Unknown {
                continue;
            }
            let id = AssetId::from(entry.id.clone());
            let virtual_path = PathBuf::from(format!("{}!{}", path.display(), entry.path));
            self.register_path(id.clone(), entry.kind, virtual_path);
            self.package_sources.insert(
                id,
                PackageAssetSource {
                    archive_index,
                    asset_id: entry.id.clone(),
                },
            );
            count += 1;
        }

        self.package_archives.push(archive);
        Ok(count)
    }

    pub fn register_xp3_file(&mut self, path: impl AsRef<Path>) -> Result<usize> {
        self.register_xp3_file_with_options(path, Xp3Options::default())
    }

    pub fn register_xp3_file_with_options(
        &mut self,
        path: impl AsRef<Path>,
        options: Xp3Options,
    ) -> Result<usize> {
        let path = path.as_ref();
        let archive = Xp3Archive::from_file_with_options(path, options)?;
        let archive_index = self.xp3_archives.len();
        let mut count = 0;

        for entry in archive.entries() {
            let asset_type = asset_type_from_path(&entry.name);
            if asset_type == AssetType::Unknown {
                continue;
            }
            let id = AssetId::from(asset_id_from_path(&entry.name));
            let virtual_path = PathBuf::from(format!("{}!{}", path.display(), entry.name));
            self.register_path(id.clone(), asset_type, virtual_path);
            self.xp3_sources.insert(
                id,
                Xp3AssetSource {
                    archive_index,
                    entry_name: entry.name.clone(),
                },
            );
            count += 1;
        }

        self.xp3_archives.push(archive);
        Ok(count)
    }

    pub fn load_texture(&self, id: impl Into<AssetId>) -> Result<TextureAsset> {
        let id = id.into();
        if let Some(bytes) = self.load_package_bytes(&id)? {
            return TextureAsset::from_bytes(&bytes)
                .with_context(|| format!("failed to load package texture `{}`", id.0));
        }
        if let Some(bytes) = self.load_xp3_bytes(&id)? {
            return TextureAsset::from_bytes(&bytes)
                .with_context(|| format!("failed to load XP3 texture `{}`", id.0));
        }

        let (id, path) = self.texture_path(id)?;
        TextureAsset::from_file(&path).with_context(|| format!("failed to load texture `{}`", id.0))
    }

    pub fn load_asset_bytes(&self, id: impl Into<AssetId>) -> Result<Vec<u8>> {
        let id = id.into();
        if let Some(bytes) = self.load_package_bytes(&id)? {
            return Ok(bytes);
        }
        if let Some(bytes) = self.load_xp3_bytes(&id)? {
            return Ok(bytes);
        }
        let record = self
            .records
            .get(&id)
            .with_context(|| format!("asset `{}` is not registered", id.0))?;
        std::fs::read(&record.path).with_context(|| {
            format!(
                "failed to read asset `{}` from {}",
                id.0,
                record.path.display()
            )
        })
    }

    pub fn load_texture_cached(&mut self, id: impl Into<AssetId>) -> Result<TextureAsset> {
        let id = id.into();
        if let Some(texture) = self.texture_cache.get(&id).cloned() {
            self.touch_texture(&id);
            return Ok(texture);
        }

        let texture = self.load_texture(id.clone())?;
        self.insert_cached_texture(id, texture.clone());
        Ok(texture)
    }

    pub fn load_texture_async(&self, id: impl Into<AssetId>) -> Result<AsyncTextureLoad> {
        let id = id.into();
        if let Some(source) = self.package_sources.get(&id).cloned() {
            let archive = self
                .package_archives
                .get(source.archive_index)
                .cloned()
                .with_context(|| format!("package archive for `{}` is not registered", id.0))?;
            let thread_id = id.clone();
            let handle = std::thread::spawn(move || {
                let bytes = archive.read_asset(&source.asset_id)?;
                TextureAsset::from_bytes(&bytes)
                    .with_context(|| format!("failed to load package texture `{}`", thread_id.0))
            });
            return Ok(AsyncTextureLoad { id, handle });
        }

        if let Some(source) = self.xp3_sources.get(&id).cloned() {
            let archive = self
                .xp3_archives
                .get(source.archive_index)
                .cloned()
                .with_context(|| format!("XP3 archive for `{}` is not registered", id.0))?;
            let thread_id = id.clone();
            let handle = std::thread::spawn(move || {
                let bytes = archive.read_file(&source.entry_name)?;
                TextureAsset::from_bytes(&bytes)
                    .with_context(|| format!("failed to load XP3 texture `{}`", thread_id.0))
            });
            return Ok(AsyncTextureLoad { id, handle });
        }

        let (id, path) = self.texture_path(id)?;
        let thread_id = id.clone();
        let handle = std::thread::spawn(move || {
            TextureAsset::from_file(&path)
                .with_context(|| format!("failed to load texture `{}`", thread_id.0))
        });
        Ok(AsyncTextureLoad { id, handle })
    }

    pub fn register_textures_from_dir(&mut self, root: impl AsRef<Path>) -> Result<usize> {
        let root = root.as_ref();
        let mut count = 0;
        if !root.exists() {
            return Ok(0);
        }

        for entry in std::fs::read_dir(root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                count += self.register_textures_from_dir(path)?;
                continue;
            }

            let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
                continue;
            };
            if !matches!(
                extension.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "webp"
            ) {
                continue;
            }

            if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                self.register_texture(stem.to_owned(), path);
                count += 1;
            }
        }

        Ok(count)
    }

    fn texture_path(&self, id: AssetId) -> Result<(AssetId, PathBuf)> {
        let Some(record) = self.records.get(&id) else {
            bail!("texture asset `{}` is not registered", id.0);
        };
        if record.asset_type != AssetType::Texture {
            bail!("asset `{}` is not a texture", id.0);
        }
        Ok((id, record.path.clone()))
    }

    fn load_xp3_bytes(&self, id: &AssetId) -> Result<Option<Vec<u8>>> {
        let Some(source) = self.xp3_sources.get(id) else {
            return Ok(None);
        };
        let archive = self
            .xp3_archives
            .get(source.archive_index)
            .with_context(|| format!("XP3 archive for `{}` is not registered", id.0))?;
        archive
            .read_file(&source.entry_name)
            .map(Some)
            .with_context(|| format!("failed to read XP3 asset `{}`", id.0))
    }

    fn load_package_bytes(&self, id: &AssetId) -> Result<Option<Vec<u8>>> {
        let Some(source) = self.package_sources.get(id) else {
            return Ok(None);
        };
        let archive = self
            .package_archives
            .get(source.archive_index)
            .with_context(|| format!("package archive for `{}` is not registered", id.0))?;
        archive
            .read_asset(&source.asset_id)
            .map(Some)
            .with_context(|| format!("failed to read package asset `{}`", id.0))
    }

    fn insert_cached_texture(&mut self, id: AssetId, texture: TextureAsset) {
        self.texture_cache.insert(id.clone(), texture);
        self.touch_texture(&id);
        self.evict_textures_to_capacity();
    }

    fn touch_texture(&mut self, id: &AssetId) {
        self.texture_lru.retain(|cached_id| cached_id != id);
        self.texture_lru.push_back(id.clone());
    }

    fn evict_textures_to_capacity(&mut self) {
        let Some(capacity) = self.texture_cache_capacity else {
            return;
        };

        while self.texture_cache.len() > capacity {
            let Some(oldest) = self.texture_lru.pop_front() else {
                break;
            };
            self.texture_cache.remove(&oldest);
        }
    }
}

fn asset_id_from_path(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(path)
        .to_owned()
}

fn asset_type_from_path(path: &str) -> AssetType {
    match Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png" | "jpg" | "jpeg" | "webp") => AssetType::Texture,
        Some("ogg" | "wav" | "mp3" | "flac") => AssetType::Audio,
        Some("szs" | "ks" | "tjs") => AssetType::Script,
        Some("ttf" | "otf") => AssetType::Font,
        Some(_) => AssetType::Data,
        None => AssetType::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{write::ZlibEncoder, Compression};
    use std::io::Write;

    #[test]
    fn registers_texture_paths() {
        let mut assets = AssetManager::default();
        assets.register_texture("bg", "assets/bg.png");

        let record = assets.get(&AssetId::from("bg")).unwrap();
        assert_eq!(record.asset_type, AssetType::Texture);
        assert_eq!(record.path, PathBuf::from("assets/bg.png"));
    }

    #[test]
    fn registers_texture_directory_recursively() {
        let mut root = std::env::temp_dir();
        root.push(format!(
            "suzu-assets-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let nested = root.join("bg");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("school.png"), []).unwrap();
        std::fs::write(root.join("ignore.txt"), []).unwrap();

        let mut assets = AssetManager::default();
        let count = assets.register_textures_from_dir(&root).unwrap();

        assert_eq!(count, 1);
        assert_eq!(
            assets.get(&AssetId::from("school")).unwrap().path,
            nested.join("school.png")
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn registers_assets_from_manifest_file() {
        let root = test_dir("suzu-assets-manifest-register");
        let asset_root = root.join("assets");
        std::fs::create_dir_all(asset_root.join("bg")).unwrap();
        let manifest_path = root.join("manifest.json");
        std::fs::write(
            &manifest_path,
            format!(
                r#"{{
  "format_version": 1,
  "generated_at_unix_ms": 1,
  "root": "{}",
  "assets": [
    {{ "id": "bg/school", "path": "bg/school.png", "kind": "texture", "bytes": 3 }},
    {{ "id": "voice/eileen", "path": "voice/eileen.ogg", "kind": "audio", "bytes": 5 }},
    {{ "id": "misc/raw", "path": "misc/raw.bin", "kind": "unknown", "bytes": 9 }}
  ]
}}"#,
                asset_root.to_string_lossy().replace('\\', "\\\\")
            ),
        )
        .unwrap();

        let mut assets = AssetManager::default();
        let count = assets.register_manifest_file(&manifest_path).unwrap();

        assert_eq!(count, 2);
        assert_eq!(assets.len(), 2);
        assert_eq!(
            assets.get(&AssetId::from("bg/school")).unwrap().path,
            asset_root.join("bg").join("school.png")
        );
        assert_eq!(
            assets
                .get(&AssetId::from("voice/eileen"))
                .unwrap()
                .asset_type,
            AssetType::Audio
        );
        assert!(assets.get(&AssetId::from("misc/raw")).is_none());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn cached_texture_loader_reuses_and_evicts_lru_entries() {
        let root = test_dir("suzu-assets-cache");
        std::fs::create_dir_all(&root).unwrap();
        let first = root.join("first.png");
        let second = root.join("second.png");
        write_test_png(&first, [1, 2, 3, 255]);
        write_test_png(&second, [4, 5, 6, 255]);

        let mut assets = AssetManager::default();
        assets.register_texture("first", &first);
        assets.register_texture("second", &second);
        assets.set_texture_cache_capacity(1);

        assert_eq!(
            assets.load_texture_cached("first").unwrap().rgba,
            vec![1, 2, 3, 255]
        );
        assert_eq!(assets.cached_texture_count(), 1);
        assert_eq!(
            assets.load_texture_cached("second").unwrap().rgba,
            vec![4, 5, 6, 255]
        );
        assert_eq!(assets.cached_texture_count(), 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn async_texture_loader_decodes_on_background_thread() {
        let root = test_dir("suzu-assets-async");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("async.png");
        write_test_png(&path, [9, 8, 7, 255]);

        let mut assets = AssetManager::default();
        assets.register_texture("async", path);

        let load = assets.load_texture_async("async").unwrap();
        assert_eq!(load.id(), &AssetId::from("async"));
        assert_eq!(load.wait().unwrap().rgba, vec![9, 8, 7, 255]);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn registers_and_loads_texture_from_xp3() {
        let root = test_dir("suzu-assets-xp3");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("data.xp3");
        let png = test_png_bytes([11, 22, 33, 255]);
        write_test_xp3(&path, "image/bg_school.png", &png);

        let mut assets = AssetManager::default();
        let count = assets.register_xp3_file(&path).unwrap();

        assert_eq!(count, 1);
        assert_eq!(
            assets.get(&AssetId::from("bg_school")).unwrap().asset_type,
            AssetType::Texture
        );
        assert_eq!(
            assets.load_texture("bg_school").unwrap().rgba,
            vec![11, 22, 33, 255]
        );

        let _ = std::fs::remove_dir_all(root);
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
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    fn write_test_png(path: &Path, rgba: [u8; 4]) {
        image::RgbaImage::from_raw(1, 1, rgba.to_vec())
            .unwrap()
            .save(path)
            .unwrap();
    }

    fn test_png_bytes(rgba: [u8; 4]) -> Vec<u8> {
        let image = image::RgbaImage::from_raw(1, 1, rgba.to_vec()).unwrap();
        let mut cursor = std::io::Cursor::new(Vec::new());
        image
            .write_to(&mut cursor, image::ImageFormat::Png)
            .unwrap();
        cursor.into_inner()
    }

    fn write_test_xp3(path: &Path, name: &str, data: &[u8]) {
        let packed_index;
        let segment_offset = 0x13_u64;
        let index_offset = segment_offset + data.len() as u64;
        let index = build_xp3_index(name, data.len() as u64, segment_offset);
        {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&index).unwrap();
            packed_index = encoder.finish().unwrap();
        }

        let mut bytes = crate::xp3_magic().to_vec();
        bytes.extend_from_slice(&index_offset.to_le_bytes());
        bytes.extend_from_slice(data);
        bytes.push(1);
        bytes.extend_from_slice(&(packed_index.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&(index.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&packed_index);
        std::fs::write(path, bytes).unwrap();
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
}
