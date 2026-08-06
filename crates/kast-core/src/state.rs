use kast_event::Event;

use crate::AppContext;

/// Application state trait representing the different lifecycle hooks.
///
/// Implementors can override any of these methods to participate in the
/// application's lifecycle. All methods have empty default implementations so
/// you only need to implement the callbacks you care about.
///
/// # Lifecycle Flow
///
/// 2. **`on_init`** - Called once when the app starts
/// 3. **Main Loop:**
///    - **`on_event`** - Called for each incoming event (window, input, etc.)
///    - **`on_update`** - Called each frame to advance game logic
///    - **`on_render`** - Called each frame to perform rendering
/// 4. **`on_exit`** - Called when the application is about to shut down
pub trait AppState {
    /// Called when the event loop starts or resumes (after OS suspension).
    fn on_resume(&mut self, _context: &mut AppContext) {}

    /// Called when the application is suspended.
    fn on_suspend(&mut self, _context: &mut AppContext) {}

    /// Called once when the app starts.
    ///
    /// This is where you should initialize your game state, load initial
    /// resources, and set up any systems you need.
    fn on_init(&mut self, _context: &mut AppContext) {}

    /// Called each frame to update game logic.
    ///
    /// This runs before rendering and is where you should update entity
    /// positions, process AI, handle physics, etc.
    fn on_update(&mut self, _context: &mut AppContext) {}

    /// Called each frame to perform rendering.
    ///
    /// This runs after update and is where you should submit draw calls,
    /// render sprites, UI, etc.
    fn on_render(&mut self, _context: &mut AppContext) {}

    /// Called for incoming events (window, input, etc).
    ///
    /// Events include keyboard/mouse input, window resize, focus changes,
    /// and custom application events.
    fn on_event(&mut self, _context: &mut AppContext, _event: &Event) {}

    /// Called when the application is about to shut down.
    ///
    /// This is where you can save state, clean up resources, or perform
    /// any final tasks before the app exits.
    fn on_exit(&mut self, _context: &mut AppContext) {}
}

/// A no-op application state you can use as a placeholder while sketching.
///
/// This implements all `AppState` methods as empty functions, useful for
/// testing the engine or as a starting point for prototyping.
#[derive(Default)]
pub struct EmptyState;

impl AppState for EmptyState {}
