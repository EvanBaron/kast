use kast_renderer::Renderer;
use kast_windowing::{WindowConfig, WindowManager};

use crate::{App, AppContext, AppState, state::EmptyState};

/// Builder for configuring an `App` before running it.
///
/// The builder pattern allows you to set various configuration options before constructing the final application.
pub struct AppBuilder {
    window_configs: Vec<WindowConfig>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            window_configs: Vec::new(),
        }
    }
}

impl AppBuilder {
    /// Add a window configuration.
    ///
    /// You can call this multiple times to create multiple windows.
    pub fn with_window(mut self, window_config: WindowConfig) -> Self {
        self.window_configs.push(window_config);

        self
    }

    /// Build the final `App` with a specific state.
    ///
    /// If no windows were configured, a default window is created
    /// using the specified title.
    pub fn build_with<S: AppState + 'static>(mut self, state: S) -> App {
        let mut window_manager = WindowManager::new();

        if self.window_configs.is_empty() {
            self.window_configs.push(WindowConfig::default())
        }

        for config in self.window_configs.into_iter() {
            window_manager.queue_window(config);
        }

        App::new(
            Box::new(state),
            AppContext {
                window_manager,
                renderer: Renderer::new(),
                exit_requested: false,
            },
        )
    }

    /// Build the app with an empty state (useful for testing/prototyping).
    pub fn build(self) -> App {
        self.build_with(EmptyState)
    }
}
