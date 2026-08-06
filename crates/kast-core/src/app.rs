use kast_event::{Event, WindowEvent, WindowEventPayload};
use kast_graphics::VulkanContext;
use kast_windowing::{EventLoopHandler, WindowManager};

use crate::{AppBuilder, AppContext, AppState};

/// The main application container.
///
/// `App` manages the application lifecycle, coordinating between the user's
/// state, the windowing system, and various subsystems through the context.
pub struct App {
    state: Box<dyn AppState>,
    context: AppContext,
    exit_requested: bool,
}

impl App {
    pub(crate) fn new(state: Box<dyn AppState>, context: AppContext) -> Self {
        Self {
            state,
            context,
            exit_requested: false,
        }
    }

    /// Construct the builder for configuring an `App`.
    ///
    /// This is the canonical public entrypoint to configure and build an
    /// application.
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    /// Run the app main loop.
    ///
    /// This consumes the `App` and delegates to the windowing crate to select
    /// and run the appropriate backend.
    pub fn run(mut self) {
        self.state.on_init(&mut self.context);

        let window_manager = core::mem::take(&mut self.context.window_manager);
        window_manager.run(self);
    }

    /// Helper to temporarily provide window manager access to the context.
    fn with_context<F>(&mut self, window_manager: &mut WindowManager, f: F)
    where
        F: FnOnce(&mut dyn AppState, &mut AppContext),
    {
        core::mem::swap(&mut self.context.window_manager, window_manager);
        f(&mut *self.state, &mut self.context);
        core::mem::swap(&mut self.context.window_manager, window_manager);
    }

    /// This can't happen at `run()` time: the winit window (and the raw window
    /// handle a graphics backend needs) is only created once the event loop is
    /// active, which is when `on_resume` first fires.
    fn init_renderer(&mut self, window_manager: &WindowManager) {
        if self.context.renderer.is_ready() {
            return;
        }

        let Some(window) = window_manager.windows.values().next() else {
            return;
        };
        let Some(winit_window) = window.raw_window() else {
            return;
        };

        let size = &window.config().size;
        match VulkanContext::new("kast", winit_window, size.width, size.height) {
            Ok(context) => self.context.renderer.attach(Box::new(context)),
            Err(error) => eprintln!("Failed to initialize renderer: {error}"),
        }
    }
}

/// Implement the event-loop handler trait so `WindowManager` can call into the
/// `App` for events and idle ticks.
impl EventLoopHandler for App {
    fn on_resume(&mut self, window_manager: &mut WindowManager) {
        self.init_renderer(window_manager);

        self.with_context(window_manager, |state, context| {
            state.on_resume(context);
        });
    }

    fn on_suspend(&mut self, window_manager: &mut WindowManager) {
        self.with_context(window_manager, |state, context| {
            state.on_suspend(context);
        });
    }

    fn on_event(&mut self, event: Event, window_manager: &mut WindowManager) {
        if let Event::Window(WindowEvent {
            payload: WindowEventPayload::Resized(width, height),
            ..
        }) = &event
        {
            self.context.renderer.resize(*width, *height);
        }

        self.with_context(window_manager, |state, context| {
            state.on_event(context, &event);
        });
    }

    fn on_update(&mut self, window_manager: &mut WindowManager) {
        self.with_context(window_manager, |state, context| {
            state.on_update(context);
        });
    }

    fn on_render(&mut self, window_manager: &mut WindowManager) {
        self.with_context(window_manager, |state, context| {
            if !context.renderer.begin_frame() {
                return;
            }

            state.on_render(context);

            context.renderer.end_frame();
        });
    }

    fn request_exit(&mut self, window_manager: &mut WindowManager) {
        self.with_context(window_manager, |state, context| {
            state.on_exit(context);
        });

        self.exit_requested = true;
    }

    fn should_exit(&self) -> bool {
        self.exit_requested || self.context.should_exit()
    }
}
