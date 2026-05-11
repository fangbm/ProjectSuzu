use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self::rgba(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::rgba(0.0, 0.0, 0.0, 1.0);
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Rect {
    pub origin: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Vec2::new(x, y),
            size: Vec2::new(width, height),
        }
    }

    pub fn contains(self, point: Vec2) -> bool {
        point.x >= self.origin.x
            && point.y >= self.origin.y
            && point.x <= self.origin.x + self.size.x
            && point.y <= self.origin.y + self.size.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Affine2 {
    pub matrix: [f32; 6],
}

impl Affine2 {
    pub const IDENTITY: Self = Self {
        matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    pub fn translation(offset: Vec2) -> Self {
        Self {
            matrix: [1.0, 0.0, 0.0, 1.0, offset.x, offset.y],
        }
    }
}
