use std::path::Path;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureAsset {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl TextureAsset {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let image = image::open(path)
            .with_context(|| format!("failed to decode texture {}", path.display()))?
            .to_rgba8();
        let (width, height) = image.dimensions();

        Ok(Self {
            width,
            height,
            rgba: image.into_raw(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn decodes_png_texture() {
        let mut path = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("suzu-texture-{unique}.png"));

        let image = image::RgbaImage::from_raw(1, 1, vec![255, 128, 64, 255]).unwrap();
        image.save(&path).unwrap();

        let texture = TextureAsset::from_file(&path).unwrap();
        assert_eq!(texture.width, 1);
        assert_eq!(texture.height, 1);
        assert_eq!(texture.rgba, vec![255, 128, 64, 255]);

        let _ = std::fs::remove_file(path);
    }
}
