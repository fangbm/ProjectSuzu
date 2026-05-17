use std::{sync::Arc, time::Instant};

use anyhow::{Context, Result};
use suzu_core::Vec2;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

use crate::WindowConfig;

use super::{
    input::{DesktopApp, DesktopInputEvent},
    window::window_attributes,
    GpuClearRenderer,
};

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
    cursor_position: Option<Vec2>,
}

impl<A> DesktopRunner<A> {
    fn new(config: WindowConfig, app: A) -> Self {
        Self {
            config,
            app,
            window: None,
            renderer: None,
            last_frame_at: None,
            cursor_position: None,
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

        let window = Arc::new(
            event_loop
                .create_window(window_attributes(&self.config))
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
            WindowEvent::CursorMoved { position, .. } => {
                let position = Vec2::new(position.x as f32, position.y as f32);
                self.cursor_position = Some(position);
                self.app.input(DesktopInputEvent::PointerMove { position });
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
                if let Some(position) = self.cursor_position {
                    self.app.input(DesktopInputEvent::PointerDown { position });
                } else {
                    self.app.input(DesktopInputEvent::Confirm);
                }
                window.request_redraw();
            }
            WindowEvent::Touch(touch) if touch.phase == TouchPhase::Started => {
                self.app.input(DesktopInputEvent::PointerDown {
                    position: Vec2::new(touch.location.x as f32, touch.location.y as f32),
                });
                window.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(position) => position.y as f32,
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
