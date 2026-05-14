use std::path::{Path, PathBuf};

use suzu_asset::AssetType;

pub fn asset_type_from_path(path: &str) -> AssetType {
    match Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png" | "jpg" | "jpeg" | "webp") => AssetType::Texture,
        Some("ogg" | "wav" | "mp3" | "flac") => AssetType::Audio,
        Some("szs" | "ks" | "tjs" | "txt") => AssetType::Script,
        Some("ttf" | "otf") => AssetType::Font,
        Some(_) => AssetType::Data,
        None => AssetType::Unknown,
    }
}

pub fn asset_id_from_path(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(path)
        .to_owned()
}

pub fn default_krkr_output_path(root: &Path) -> PathBuf {
    let folder_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(|name| format!("{name}-suzu-migration"))
        .unwrap_or_else(|| "suzu-migration".to_owned());
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .map(|home| {
            home.join("Documents")
                .join("ProjectSuzu Migrations")
                .join(&folder_name)
        })
        .unwrap_or_else(|| root.join(folder_name))
}

pub fn xp3_path_from_input(input: &str) -> Result<PathBuf, String> {
    let cleaned = clean_path_input(input);
    if cleaned.is_empty() {
        return Err("Enter an XP3 path first.".to_owned());
    }
    let path = PathBuf::from(cleaned);
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xp3"))
    {
        Ok(path)
    } else {
        Err("The selected file is not an .xp3 archive.".to_owned())
    }
}

pub fn clean_path_input(input: &str) -> String {
    let mut value = input.trim().trim_matches(['"', '\'']).trim().to_owned();
    if let Some(rest) = value.strip_prefix("file:///") {
        value = rest.replace('/', "\\");
    } else if let Some(rest) = value.strip_prefix("file://") {
        value = rest.replace('/', "\\");
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_quoted_path() {
        assert_eq!(
            clean_path_input(r#""D:\games\Suzu\data.xp3""#),
            r"D:\games\Suzu\data.xp3"
        );
    }

    #[test]
    fn recognizes_xp3_paths() {
        assert!(xp3_path_from_input(r"D:\games\Suzu\data.xp3").is_ok());
        assert!(xp3_path_from_input(r"D:\games\Suzu\data.zip").is_err());
    }
}
