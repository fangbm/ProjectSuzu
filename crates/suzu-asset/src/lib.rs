pub mod archive;
pub mod krkr;
pub mod manager;
pub mod manifest;
pub mod texture;
pub mod types;
pub mod xp3;
pub mod xp3_plugin;

pub use archive::{archive_magic, checksum64, PackageArchive};
pub use krkr::{probe_krkr_directory, KrkrArchiveReport, KrkrCompatibilityReport};
pub use manager::{AssetManager, AsyncTextureLoad};
pub use manifest::{AssetCompression, AssetManifestEntry, PackageManifest};
pub use texture::TextureAsset;
pub use types::{AssetId, AssetType, Handle};
pub use xp3::{
    xp3_magic, Xp3Archive, Xp3Entry, Xp3Options, Xp3Plugin, Xp3PluginScheme, Xp3Segment,
};
pub use xp3_plugin::Xp3PluginModule;
