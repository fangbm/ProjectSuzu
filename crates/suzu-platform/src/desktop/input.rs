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
    Scroll { delta: f32 },
}
