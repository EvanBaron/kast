use kast_graphics::GraphicsContext;

pub struct Renderer {
    context: Option<Box<dyn GraphicsContext>>,
}

impl Renderer {
    pub fn new() -> Self {
        Self { context: None }
    }

    /// Whether a backend graphics context has been attached yet.
    pub fn is_ready(&self) -> bool {
        self.context.is_some()
    }

    /// Attaches an already-initialized backend graphics context.
    ///
    /// Building the context is windowing- and backend-specific (it needs a live
    /// window/surface handle), so that's done by whoever owns both the window and
    /// the graphics backend (kast-core) — the renderer only ever deals with an
    /// active `GraphicsContext`.
    pub fn attach(&mut self, context: Box<dyn GraphicsContext>) {
        self.context = Some(context);
    }

    /// Direct access to the backend for creating resources and submitting draw calls.
    pub fn context_mut(&mut self) -> Option<&mut (dyn GraphicsContext + 'static)> {
        self.context.as_deref_mut()
    }

    pub fn begin_frame(&mut self) -> bool {
        match &mut self.context {
            Some(context) => match context.begin_frame() {
                Ok(()) => true,
                Err(error) => {
                    eprintln!("Renderer: begin_frame failed: {error}");
                    false
                }
            },
            None => false,
        }
    }

    pub fn end_frame(&mut self) {
        if let Some(context) = &mut self.context {
            if let Err(error) = context.end_frame() {
                eprintln!("Renderer: end_frame failed: {error}");
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(context) = &mut self.context {
            context.resize(width, height);
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
