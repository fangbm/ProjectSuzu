use bytemuck::{Pod, Zeroable};
use suzu_core::Vec2;
use wgpu::util::DeviceExt;

use super::frame::{FrameBlendMode, FrameSprite};

pub(super) struct SpriteDrawBuffer {
    pub(super) buffer: wgpu::Buffer,
    pub(super) blend_mode: FrameBlendMode,
}

pub(super) fn create_sprite_draw_buffer(
    device: &wgpu::Device,
    sprite: &FrameSprite,
    surface_width: u32,
    surface_height: u32,
) -> SpriteDrawBuffer {
    let vertices = sprite_vertices(sprite, surface_width, surface_height);
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("suzu-sprite-vertices"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    SpriteDrawBuffer {
        buffer,
        blend_mode: sprite.blend_mode,
    }
}

pub(super) fn sprite_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    SpriteVertex::layout()
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SpriteVertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

impl SpriteVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

fn sprite_vertices(
    sprite: &FrameSprite,
    surface_width: u32,
    surface_height: u32,
) -> [SpriteVertex; 6] {
    let bounds = sprite.bounds;
    let center = Vec2::new(
        bounds.origin.x + bounds.size.x * 0.5,
        bounds.origin.y + bounds.size.y * 0.5,
    );
    let half_size = Vec2::new(
        bounds.size.x * sprite.scale.x * 0.5,
        bounds.size.y * sprite.scale.y * 0.5,
    );
    let (left_u, right_u) = if sprite.flip_x {
        (1.0, 0.0)
    } else {
        (0.0, 1.0)
    };
    let corners = [
        (Vec2::new(-half_size.x, -half_size.y), [left_u, 0.0]),
        (Vec2::new(half_size.x, -half_size.y), [right_u, 0.0]),
        (Vec2::new(half_size.x, half_size.y), [right_u, 1.0]),
        (Vec2::new(-half_size.x, half_size.y), [left_u, 1.0]),
    ];
    let color = [
        sprite.tint.r,
        sprite.tint.g,
        sprite.tint.b,
        sprite.tint.a * sprite.opacity.clamp(0.0, 1.0),
    ];
    let transformed = corners.map(|(corner, uv)| {
        let rotated = rotate(corner, sprite.rotation);
        let position = Vec2::new(center.x + rotated.x, center.y + rotated.y);
        SpriteVertex {
            position: [
                to_clip_x(position.x, surface_width),
                to_clip_y(position.y, surface_height),
            ],
            uv,
            color,
        }
    });

    [
        transformed[0],
        transformed[1],
        transformed[2],
        transformed[0],
        transformed[2],
        transformed[3],
    ]
}

fn rotate(value: Vec2, radians: f32) -> Vec2 {
    let (sin, cos) = radians.sin_cos();
    Vec2::new(value.x * cos - value.y * sin, value.x * sin + value.y * cos)
}

fn to_clip_x(value: f32, surface_width: u32) -> f32 {
    value / surface_width as f32 * 2.0 - 1.0
}

fn to_clip_y(value: f32, surface_height: u32) -> f32 {
    1.0 - value / surface_height as f32 * 2.0
}

#[cfg(test)]
mod tests {
    use suzu_core::{Color, Rect};

    use super::*;

    #[test]
    fn sprite_vertices_apply_opacity_and_scale() {
        let sprite = FrameSprite::solid(
            "test",
            Rect::new(10.0, 10.0, 20.0, 20.0),
            Color::rgba(1.0, 0.5, 0.25, 0.8),
            0,
        )
        .with_opacity(0.5)
        .with_scale(Vec2::new(2.0, 1.0));

        let vertices = sprite_vertices(&sprite, 100, 100);

        assert_eq!(vertices[0].color, [1.0, 0.5, 0.25, 0.4]);
        assert_close(vertices[0].position[0], -1.0);
        assert_close(vertices[1].position[0], -0.2);
    }

    #[test]
    fn sprite_vertices_rotate_around_center() {
        let sprite = FrameSprite::solid("test", Rect::new(40.0, 40.0, 20.0, 20.0), Color::WHITE, 0)
            .with_rotation(std::f32::consts::FRAC_PI_2);

        let vertices = sprite_vertices(&sprite, 100, 100);

        assert_close(vertices[0].position[0], 0.2);
        assert_close(vertices[0].position[1], 0.2);
    }

    #[test]
    fn sprite_vertices_flip_horizontal_uvs() {
        let sprite = FrameSprite::solid("test", Rect::new(40.0, 40.0, 20.0, 20.0), Color::WHITE, 0)
            .with_flip_x(true);

        let vertices = sprite_vertices(&sprite, 100, 100);

        assert_eq!(vertices[0].uv, [1.0, 0.0]);
        assert_eq!(vertices[1].uv, [0.0, 0.0]);
        assert_eq!(vertices[2].uv, [0.0, 1.0]);
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {actual} to be close to {expected}"
        );
    }
}
