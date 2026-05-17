use suzu_core::{Color, Rect, Vec2};

pub struct DesktopFrame {
    pub clear_color: Color,
    pub textures: Vec<FrameTexture>,
    pub sprites: Vec<FrameSprite>,
    pub texts: Vec<FrameText>,
}

impl Default for DesktopFrame {
    fn default() -> Self {
        Self {
            clear_color: Color::rgba(0.05, 0.055, 0.075, 1.0),
            textures: Vec::new(),
            sprites: Vec::new(),
            texts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameText {
    pub content: String,
    pub bounds: Rect,
    pub color: Color,
    pub z_index: i32,
}

impl FrameText {
    pub fn new(content: impl Into<String>, bounds: Rect, color: Color, z_index: i32) -> Self {
        Self {
            content: content.into(),
            bounds,
            color,
            z_index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameTexture {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl FrameTexture {
    pub fn new(id: impl Into<String>, width: u32, height: u32, rgba: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            width,
            height,
            rgba,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameSprite {
    pub texture_id: String,
    pub bounds: Rect,
    pub tint: Color,
    pub opacity: f32,
    pub scale: Vec2,
    pub rotation: f32,
    pub flip_x: bool,
    pub blend_mode: FrameBlendMode,
    pub z_index: i32,
}

impl FrameSprite {
    pub fn solid(texture_id: impl Into<String>, bounds: Rect, tint: Color, z_index: i32) -> Self {
        Self {
            texture_id: texture_id.into(),
            bounds,
            tint,
            opacity: 1.0,
            scale: Vec2::ONE,
            rotation: 0.0,
            flip_x: false,
            blend_mode: FrameBlendMode::Normal,
            z_index,
        }
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_rotation(mut self, radians: f32) -> Self {
        self.rotation = radians;
        self
    }

    pub fn with_flip_x(mut self, flip_x: bool) -> Self {
        self.flip_x = flip_x;
        self
    }

    pub fn with_blend_mode(mut self, blend_mode: FrameBlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameBlendMode {
    Normal,
    Add,
    Multiply,
    Screen,
}
