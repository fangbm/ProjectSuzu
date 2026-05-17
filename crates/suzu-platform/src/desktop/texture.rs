use suzu_core::Color;

use super::frame::FrameTexture;

#[derive(Debug)]
pub(super) struct SpriteTexture {
    pub(super) view: wgpu::TextureView,
}

impl SpriteTexture {
    pub(super) fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
        source: &FrameTexture,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: source.width,
                height: source.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &source.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(source.width * 4),
                rows_per_image: Some(source.height),
            },
            wgpu::Extent3d {
                width: source.width,
                height: source.height,
                depth_or_array_layers: 1,
            },
        );
        Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
        }
    }

    pub(super) fn solid_color(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
        color: Color,
    ) -> Self {
        let rgba = [
            channel(color.r),
            channel(color.g),
            channel(color.b),
            channel(color.a),
        ];
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
        }
    }
}

pub(super) fn channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub(super) fn texture_color(texture_id: &str, tint: Color) -> Color {
    if tint != Color::WHITE {
        return tint;
    }

    match texture_id {
        "bg_school_evening" | "background" => Color::rgba(0.22, 0.28, 0.38, 1.0),
        "eileen" | "character" => Color::rgba(0.86, 0.68, 0.74, 1.0),
        texture_id if texture_id.starts_with("eileen_") => Color::rgba(0.86, 0.68, 0.74, 1.0),
        _ => Color::WHITE,
    }
}
