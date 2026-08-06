use kast_event::WindowId;
use std::sync::Arc;
use winit::window::Window as WinitWindow;

use crate::WindowConfig;

#[derive(Clone, Debug)]
pub struct Window {
    pub id: WindowId,
    pub config: WindowConfig,
    pub(crate) inner: Option<Arc<WinitWindow>>,
}

impl Window {
    /// Construct a new `Window` container from an id and config.
    pub fn new(id: WindowId, config: WindowConfig) -> Self {
        Self {
            id,
            config,
            inner: None,
        }
    }

    /// Return the opaque window id.
    pub fn id(&self) -> WindowId {
        self.id
    }

    /// Borrow the window's configuration.
    pub fn config(&self) -> &WindowConfig {
        &self.config
    }

    /// Replace the window configuration.
    pub fn set_config(&mut self, config: WindowConfig) {
        self.config = config;
    }

    // Helper to get raw handle for rendering
    pub fn raw_window(&self) -> Option<&WinitWindow> {
        self.inner.as_deref()
    }
}
