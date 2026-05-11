use serde::{Deserialize, Serialize};
use suzu_core::{Affine2, Rect, Vec2};

use super::{BlendMode, LayerNode};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteLayer {
    pub entity_id: Option<String>,
    pub texture_id: String,
    pub position: Vec2,
    pub size: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
    pub opacity: f32,
    #[serde(default)]
    pub flip_x: bool,
    pub blend_mode: BlendMode,
    pub z_index: i32,
}

impl LayerNode for SpriteLayer {
    fn bounds(&self) -> Rect {
        Rect {
            origin: self.position,
            size: self.size,
        }
    }

    fn opacity(&self) -> f32 {
        self.opacity
    }

    fn transform(&self) -> Affine2 {
        Affine2::translation(self.position)
    }

    fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    fn z_index(&self) -> i32 {
        self.z_index
    }
}
