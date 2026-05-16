use std::path::{Path, PathBuf};

use suzu_asset::AssetType;

pub(crate) fn asset_type_from_path(path: &str) -> AssetType {
    match Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png" | "jpg" | "jpeg" | "webp" | "tlg") => AssetType::Texture,
        Some("ogg" | "wav" | "mp3" | "flac") => AssetType::Audio,
        Some("szs" | "ks" | "tjs" | "txt") => AssetType::Script,
        Some("ttf" | "otf") => AssetType::Font,
        Some(_) => AssetType::Data,
        None => AssetType::Unknown,
    }
}

pub(crate) fn asset_id_from_path(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(path)
        .to_owned()
}

pub(crate) fn xp3_path_from_input(input: &str) -> Result<PathBuf, String> {
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

pub(crate) fn clean_path_input(input: &str) -> String {
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
    fn cleans_quoted_windows_xp3_path() {
        assert_eq!(
            clean_path_input(r#""D:\games\Suzu\data.xp3""#),
            r"D:\games\Suzu\data.xp3"
        );
    }

    #[test]
    fn cleans_file_url_xp3_path() {
        assert_eq!(
            clean_path_input("file:///D:/games/Suzu/data.xp3"),
            r"D:\games\Suzu\data.xp3"
        );
    }

    #[test]
    fn rejects_non_xp3_path() {
        assert!(xp3_path_from_input(r"D:\games\Suzu\data.zip").is_err());
    }

    #[test]
    fn treats_tlg_as_texture_for_plugin_preview() {
        assert_eq!(asset_type_from_path("image/bg.tlg"), AssetType::Texture);
    }
}
