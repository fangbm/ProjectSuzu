pub mod desktop;
pub mod mobile;
pub mod web;

use serde::{Deserialize, Serialize};
use suzu_core::Vec2;

pub use desktop::{
    run_desktop, DesktopApp, DesktopFrame, DesktopInputEvent, FrameBlendMode, FrameSprite,
    FrameText, FrameTexture, GpuClearRenderer,
};
pub use mobile::{MobileBuildTarget, MobileOs};
pub use web::WebBuildTarget;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformKind {
    Desktop,
    Mobile,
    Web,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub logical_size: Vec2,
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Project Suzu".to_owned(),
            logical_size: Vec2::new(1280.0, 720.0),
            resizable: true,
        }
    }
}
