mod backend;
mod config;
mod manager;
mod window;

use kast_event::Event;

pub use config::*;
pub use manager::*;
pub use window::*;

/// Event loop handler callbacks implemented by kast-core (or the owner of the
/// application logic). `WindowManager::run` will call these methods.
///
/// The handler receives a mutable reference to the concrete `manager::WindowManager` so
/// it may inspect or mutate window state during callbacks.
pub trait EventLoopHandler {
    /// Called when the event loop starts or resumes (after OS suspension).
    fn on_resume(&mut self, window_manager: &mut WindowManager);

    /// Called when the application is suspended.
    fn on_suspend(&mut self, window_manager: &mut WindowManager);

    /// Called when a translated input/window event occurs.
    fn on_event(&mut self, event: Event, window_manager: &mut WindowManager);

    /// Called every frame to advance game logic (physics, AI, input processing).
    fn on_update(&mut self, window_manager: &mut WindowManager);

    /// Called when the OS or game requests a new frame to be drawn.
    fn on_render(&mut self, window_manager: &mut WindowManager);

    /// Request that the handler perform exit work (shutdown resources, etc).
    fn request_exit(&mut self, window_manager: &mut WindowManager);

    /// Query whether the handler has requested exit / should stop the loop.
    fn should_exit(&self) -> bool;
}
