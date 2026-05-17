use winit::{dpi::LogicalSize, window::WindowAttributes};

use crate::WindowConfig;

pub(super) fn window_attributes(config: &WindowConfig) -> WindowAttributes {
    WindowAttributes::default()
        .with_title(config.title.clone())
        .with_resizable(config.resizable)
        .with_inner_size(LogicalSize::new(
            config.logical_size.x as f64,
            config.logical_size.y as f64,
        ))
}
