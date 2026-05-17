use suzu_core::Vec2;

use super::frame::DesktopFrame;

pub trait DesktopApp {
    fn input(&mut self, _event: DesktopInputEvent) {}

    fn update(&mut self, delta_ms: u32) -> DesktopFrame;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DesktopInputEvent {
    Confirm,
    Cancel,
    MoveSelection { delta: i32 },
    PointerMove { position: Vec2 },
    PointerDown { position: Vec2 },
    Scroll { delta: f32 },
}
