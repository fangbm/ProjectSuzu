use std::{collections::HashMap, sync::Arc, time::Instant};

use anyhow::{Context, Result};
use bytemuck::{Pod, Zeroable};
use cosmic_text::{
    Attrs, Buffer, Color as TextColor, FontSystem, Metrics, Shaping, SwashCache, Wrap,
};
use suzu_core::{Color, Rect, Vec2};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

use crate::WindowConfig;

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

pub trait DesktopApp {
    fn input(&mut self, _event: DesktopInputEvent) {}

    fn update(&mut self, delta_ms: u32) -> DesktopFrame;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DesktopInputEvent {
    Confirm,
    Cancel,
    MoveSelection { delta: i32 },
    Scroll { delta: f32 },
}

pub fn run_desktop<A>(config: WindowConfig, app: A) -> Result<()>
where
    A: DesktopApp + 'static,
{
    let event_loop = EventLoop::new().context("failed to create winit event loop")?;
    let mut runner = DesktopRunner::new(config, app);
    event_loop
        .run_app(&mut runner)
        .context("desktop event loop failed")
}

struct DesktopRunner<A> {
    config: WindowConfig,
    app: A,
    window: Option<Arc<Window>>,
    renderer: Option<GpuClearRenderer>,
    last_frame_at: Option<Instant>,
}

impl<A> DesktopRunner<A> {
    fn new(config: WindowConfig, app: A) -> Self {
        Self {
            config,
            app,
            window: None,
            renderer: None,
            last_frame_at: None,
        }
    }
}

impl<A> ApplicationHandler for DesktopRunner<A>
where
    A: DesktopApp,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = WindowAttributes::default()
            .with_title(self.config.title.clone())
            .with_resizable(self.config.resizable)
            .with_inner_size(LogicalSize::new(
                self.config.logical_size.x as f64,
                self.config.logical_size.y as f64,
            ));

        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("failed to create desktop window"),
        );
        let renderer = pollster::block_on(GpuClearRenderer::new(window.clone()))
            .expect("failed to initialize wgpu renderer");

        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height);
                }
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.logical_key {
                    Key::Named(NamedKey::Enter | NamedKey::Space) => {
                        self.app.input(DesktopInputEvent::Confirm);
                        window.request_redraw();
                    }
                    Key::Named(NamedKey::Escape) => {
                        self.app.input(DesktopInputEvent::Cancel);
                        window.request_redraw();
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        self.app
                            .input(DesktopInputEvent::MoveSelection { delta: 1 });
                        window.request_redraw();
                    }
                    Key::Named(NamedKey::ArrowUp) => {
                        self.app
                            .input(DesktopInputEvent::MoveSelection { delta: -1 });
                        window.request_redraw();
                    }
                    _ => {}
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.app.input(DesktopInputEvent::Confirm);
                window.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(position) => position.y as f32,
                };
                self.app.input(DesktopInputEvent::Scroll { delta });
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta_ms = self
                    .last_frame_at
                    .map(|last_frame_at| now.saturating_duration_since(last_frame_at).as_millis())
                    .unwrap_or(0)
                    .min(u32::MAX as u128) as u32;
                self.last_frame_at = Some(now);
                let frame = self.app.update(delta_ms);
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.render(frame);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

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
            let texture = self.rasterize_text(&texture_id, &text);
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

    fn rasterize_text(&mut self, texture_id: &str, text: &FrameText) -> FrameTexture {
        let width = text.bounds.size.x.ceil().max(1.0) as u32;
        let height = text.bounds.size.y.ceil().max(1.0) as u32;
        let mut rgba = vec![0; width as usize * height as usize * 4];
        let font_size = text_font_size(text.bounds);
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(font_size, font_size * 1.35),
        );
        buffer.set_size(
            &mut self.font_system,
            Some(width as f32),
            Some(height as f32),
        );
        buffer.set_wrap(&mut self.font_system, Wrap::WordOrGlyph);
        buffer.set_text(
            &mut self.font_system,
            &text.content,
            &Attrs::new(),
            Shaping::Advanced,
        );
        buffer.draw(
            &mut self.font_system,
            &mut self.swash_cache,
            text_color(text.color),
            |x, y, _w, _h, color| {
                if x < 0 || y < 0 {
                    return;
                }
                let x = x as u32;
                let y = y as u32;
                if x >= width || y >= height {
                    return;
                }
                blend_pixel(&mut rgba, width, x, y, color.as_rgba());
            },
        );

        FrameTexture::new(texture_id, width, height, rgba)
    }

    fn create_sprite_vertex_buffer(&self, sprite: &FrameSprite) -> SpriteDrawBuffer {
        let vertices = sprite_vertices(sprite, self.config.width, self.config.height);
        use wgpu::util::DeviceExt;
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("suzu-sprite-vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        SpriteDrawBuffer {
            buffer,
            blend_mode: sprite.blend_mode,
        }
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

struct SpriteDrawBuffer {
    buffer: wgpu::Buffer,
    blend_mode: FrameBlendMode,
}

#[derive(Debug)]
struct SpriteTexture {
    view: wgpu::TextureView,
}

impl SpriteTexture {
    fn from_rgba(
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

    fn solid_color(device: &wgpu::Device, queue: &wgpu::Queue, label: &str, color: Color) -> Self {
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

fn create_sprite_pipelines(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> HashMap<FrameBlendMode, wgpu::RenderPipeline> {
    let mut pipelines = HashMap::new();
    for blend_mode in [
        FrameBlendMode::Normal,
        FrameBlendMode::Add,
        FrameBlendMode::Multiply,
        FrameBlendMode::Screen,
    ] {
        pipelines.insert(
            blend_mode,
            create_sprite_pipeline(device, format, bind_group_layout, blend_mode),
        );
    }
    pipelines
}

fn create_sprite_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
    blend_mode: FrameBlendMode,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("suzu-sprite-shader"),
        source: wgpu::ShaderSource::Wgsl(SPRITE_SHADER.into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("suzu-sprite-pipeline-layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("suzu-sprite-pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[SpriteVertex::layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(blend_state(blend_mode)),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

fn blend_state(blend_mode: FrameBlendMode) -> wgpu::BlendState {
    let alpha = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    let color = match blend_mode {
        FrameBlendMode::Normal => wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        FrameBlendMode::Add => wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
        FrameBlendMode::Multiply => wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Dst,
            dst_factor: wgpu::BlendFactor::Zero,
            operation: wgpu::BlendOperation::Add,
        },
        FrameBlendMode::Screen => wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::OneMinusDst,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
    };
    wgpu::BlendState { color, alpha }
}

fn text_font_size(bounds: Rect) -> f32 {
    bounds.size.y.mul_add(0.28, 0.0).clamp(18.0, 30.0)
}

fn text_color(color: Color) -> TextColor {
    TextColor::rgba(
        channel(color.r),
        channel(color.g),
        channel(color.b),
        channel(color.a),
    )
}

fn blend_pixel(rgba: &mut [u8], width: u32, x: u32, y: u32, source: [u8; 4]) {
    let offset = ((y * width + x) * 4) as usize;
    let src_a = source[3] as f32 / 255.0;
    if src_a <= 0.0 {
        return;
    }

    let dst = [
        rgba[offset],
        rgba[offset + 1],
        rgba[offset + 2],
        rgba[offset + 3],
    ];
    let dst_a = dst[3] as f32 / 255.0;
    let out_a = src_a + dst_a * (1.0 - src_a);
    if out_a <= 0.0 {
        return;
    }

    for channel_index in 0..3 {
        let src = source[channel_index] as f32 / 255.0;
        let dst = dst[channel_index] as f32 / 255.0;
        let out = (src * src_a + dst * dst_a * (1.0 - src_a)) / out_a;
        rgba[offset + channel_index] = channel(out);
    }
    rgba[offset + 3] = channel(out_a);
}

fn channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn texture_color(texture_id: &str, tint: Color) -> Color {
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

const SPRITE_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.uv = input.uv;
    output.color = input.color;
    return output;
}

@group(0) @binding(0) var sprite_texture: texture_2d<f32>;
@group(0) @binding(1) var sprite_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(sprite_texture, sprite_sampler, input.uv) * input.color;
}
"#;

#[cfg(test)]
mod tests {
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
