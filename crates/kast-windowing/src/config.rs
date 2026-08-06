use kast_math::Vec2;

#[derive(Debug, Clone)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

/// Supported window modes.
#[derive(Debug, Clone)]
pub enum WindowMode {
    Windowed,
    Borderless,
    Fullscreen,
}

/// Generic, backend-agnostic configuration for creating a window.
///
/// Backends should map these fields to platform-specific window creation APIs.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub size: PhysicalSize,
    pub position: Option<Vec2>,
    pub mode: WindowMode,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("Kast Window"),
            size: PhysicalSize {
                width: 800,
                height: 600,
            },
            position: None,
            mode: WindowMode::Windowed,
        }
    }
}
