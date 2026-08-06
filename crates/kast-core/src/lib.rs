pub mod app;
pub mod builder;
pub mod state;

pub use app::App;
pub use builder::AppBuilder;
use kast_renderer::Renderer;
pub use kast_windowing::*;
pub use state::AppState;

/// Commonly used types that users typically want to import.
///
/// Use this with `use kast_core::prelude::*;` to get started quickly.
pub mod prelude {
    pub use crate::{App, AppBuilder, AppContext, AppState};
    pub use kast_event::Event;
    pub use kast_graphics::{
        GraphicsContext,
        command::*,
        descriptors::*,
        enums::*,
        handle::*,
    };
    pub use kast_windowing::{PhysicalSize, WindowConfig, WindowManager, WindowMode};
}

/// The application context passed to state callbacks.
///
/// This provides access to various engine subsystems like the window manager,
/// and allows states to request application exit.
pub struct AppContext {
    pub renderer: Renderer,
    pub window_manager: WindowManager,
    pub(crate) exit_requested: bool,
}

impl AppContext {
    /// Request the application to exit gracefully.
    ///
    /// This will trigger the `on_exit` callback and then terminate the event loop.
    pub fn quit(&mut self) {
        self.exit_requested = true;
    }

    /// Check if exit has been requested.
    pub(crate) fn should_exit(&self) -> bool {
        self.exit_requested
    }
}
