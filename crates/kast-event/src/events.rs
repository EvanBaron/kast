/// A unique identifier for a window.
pub type WindowId = u64;

#[derive(Clone, Debug)]
pub enum Event {
    /// Application lifecycle events.
    App(AppEvent),

    /// Window-specific events.
    Window(WindowEvent),

    /// Input events.
    Input(InputEvent),

    /// A tick/update event.
    Tick,
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    /// The OS has suspended the app.
    Suspended,
    /// The OS has resumed the app.
    Resumed,
    /// The application is being asked to quit.
    Quit,
}

#[derive(Clone, Debug)]
pub struct WindowEvent {
    pub window_id: WindowId,
    pub payload: WindowEventPayload,
}

#[derive(Clone, Debug)]
pub enum WindowEventPayload {
    /// The window was resized to the given logical (width, height).
    Resized(u32, u32),

    /// The user requested to close the window.
    CloseRequested,

    /// The window gained or lost focus.
    Focused(bool),
}

#[derive(Clone, Debug)]
pub enum InputEvent {
    Dummy,
}
