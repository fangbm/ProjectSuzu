use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use cosmic_text::{FontSystem, SwashCache};
use suzu_core::Color;
use winit::window::Window;

use super::{
    frame::{DesktopFrame, FrameBlendMode, FrameSprite, FrameTexture},
    pipeline::create_sprite_pipelines,
    sprite::{create_sprite_draw_buffer, SpriteDrawBuffer},
    text::rasterize_text,
    texture::{texture_color, SpriteTexture},
};

pub struct GpuClearRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipelines: HashMap<FrameBlendMode, wgpu::RenderPipeline>,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    textures: HashMap<String, SpriteTexture>,
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_frame_index: u64,
}

impl GpuClearRenderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .context("failed to create wgpu surface")?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("failed to find a compatible GPU adapter")?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("suzu-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .context("failed to create wgpu device")?;

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);
        let present_mode = capabilities
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .unwrap_or(capabilities.present_modes[0]);
        let alpha_mode = capabilities.alpha_modes[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("suzu-sprite-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("suzu-sprite-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let pipelines = create_sprite_pipelines(&device, config.format, &bind_group_layout);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipelines,
            bind_group_layout,
            sampler,
            textures: HashMap::new(),
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            text_frame_index: 0,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, mut frame_data: DesktopFrame) {
        self.append_text_sprites(&mut frame_data);
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                panic!("wgpu surface is out of memory");
            }
            Err(wgpu::SurfaceError::Timeout | wgpu::SurfaceError::Other) => return,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("suzu-clear-encoder"),
            });

        frame_data.sprites.sort_by_key(|sprite| sprite.z_index);
        for texture in frame_data.textures {
            self.register_texture(texture);
        }
        let draw_calls = frame_data
            .sprites
            .iter()
            .map(|sprite| {
                let vertex_buffer = self.create_sprite_vertex_buffer(sprite);
                let bind_group = self.bind_group_for(&sprite.texture_id, sprite.tint);
                (vertex_buffer, bind_group)
            })
            .collect::<Vec<_>>();

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("suzu-clear-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: frame_data.clear_color.r as f64,
                            g: frame_data.clear_color.g as f64,
                            b: frame_data.clear_color.b as f64,
                            a: frame_data.clear_color.a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            for (vertex_buffer, bind_group) in &draw_calls {
                let pipeline = self
                    .pipelines
                    .get(&vertex_buffer.blend_mode)
                    .expect("sprite blend pipeline should exist");
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, bind_group, &[]);
                pass.set_vertex_buffer(0, vertex_buffer.buffer.slice(..));
                pass.draw(0..6, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn append_text_sprites(&mut self, frame_data: &mut DesktopFrame) {
        self.text_frame_index = self.text_frame_index.wrapping_add(1);
        let texts = std::mem::take(&mut frame_data.texts);
        for (index, text) in texts.into_iter().enumerate() {
            if text.content.trim().is_empty()
                || text.bounds.size.x <= 0.0
                || text.bounds.size.y <= 0.0
            {
                continue;
            }

            let texture_id = format!("__suzu_text_{}_{}", self.text_frame_index, index);
            let texture = rasterize_text(
                &texture_id,
                &text,
                &mut self.font_system,
                &mut self.swash_cache,
            );
            frame_data.sprites.push(FrameSprite::solid(
                texture.id.clone(),
                text.bounds,
                Color::WHITE,
                text.z_index,
            ));
            frame_data.textures.push(texture);
        }
        frame_data.sprites.sort_by_key(|sprite| sprite.z_index);
    }

    fn create_sprite_vertex_buffer(&self, sprite: &FrameSprite) -> SpriteDrawBuffer {
        create_sprite_draw_buffer(&self.device, sprite, self.config.width, self.config.height)
    }

    fn bind_group_for(&mut self, texture_id: &str, tint: Color) -> wgpu::BindGroup {
        let texture = self
            .textures
            .entry(texture_id.to_owned())
            .or_insert_with(|| {
                SpriteTexture::solid_color(
                    &self.device,
                    &self.queue,
                    texture_id,
                    texture_color(texture_id, tint),
                )
            });

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("suzu-sprite-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    fn register_texture(&mut self, texture: FrameTexture) {
        if texture.width == 0
            || texture.height == 0
            || texture.rgba.len() != texture.width as usize * texture.height as usize * 4
        {
            return;
        }

        let gpu_texture =
            SpriteTexture::from_rgba(&self.device, &self.queue, &texture.id, &texture);
        self.textures.insert(texture.id, gpu_texture);
    }
}
