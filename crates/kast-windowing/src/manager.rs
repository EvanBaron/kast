use kast_event::WindowId;
use std::{collections::HashMap, sync::Arc};
use winit::{
    dpi::PhysicalSize as WinitPhysicalSize,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window as WinitWindow,
    window::WindowId as WinitWindowId,
};

use crate::{EventLoopHandler, Window, WindowConfig, backend::WinitApp};

#[derive(Debug)]
pub struct WindowManager {
    next_id: WindowId,
    pub pending_windows: Vec<WindowId>,
    pub windows: HashMap<WindowId, Window>,
    pub id_map: HashMap<WinitWindowId, WindowId>,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowManager {
    /// Create a new window manager.
    pub fn new() -> Self {
        Self {
            next_id: 1,
            pending_windows: Vec::new(),
            windows: HashMap::new(),
            id_map: HashMap::new(),
        }
    }

    /// Queue a window to be created.
    ///
    /// The physical window will be created when the event loop is active.
    pub fn queue_window(&mut self, config: WindowConfig) -> WindowId {
        let id = self.next_id;
        self.next_id += 1;

        let window = Window::new(id, config);
        self.windows.insert(id, window);
        self.pending_windows.push(id);

        id
    }

    /// Return an immutable reference to a `Window` by id.
    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }

    /// Called by the backend when the OS is ready to create windows.
    pub(crate) fn process_pending_windows(&mut self, event_loop: &ActiveEventLoop) {
        if self.pending_windows.is_empty() {
            return;
        }

        let pending = std::mem::take(&mut self.pending_windows);

        for engine_id in pending {
            if let Some(window) = self.windows.get_mut(&engine_id) {
                let attributes = WinitWindow::default_attributes()
                    .with_title(&window.config.title)
                    .with_inner_size(WinitPhysicalSize::new(
                        window.config.size.width,
                        window.config.size.height,
                    ));

                match event_loop.create_window(attributes) {
                    Ok(winit_window) => {
                        let winit_id = winit_window.id();
                        window.inner = Some(Arc::new(winit_window));
                        self.id_map.insert(winit_id, engine_id);
                    }
                    Err(error) => {
                        eprintln!("Failed to create window {}: {}", engine_id, error);
                    }
                }
            }
        }
    }

    /// Entry point to run the application.
    pub fn run<H: EventLoopHandler + 'static>(self, handler: H) {
        let event_loop = EventLoop::new().unwrap();

        let mut app = WinitApp::new(self, handler);

        event_loop.run_app(&mut app).unwrap();
    }
}
