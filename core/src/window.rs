use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    raw_window_handle::{HasDisplayHandle, RawDisplayHandle},
    window::{Window, WindowAttributes},
};

use crate::graphics::instance::Instance;
use crate::graphics::renderer::Renderer;

const WINDOW_TITLE: &'static str = "Kast";

/// Returns the required instance extensions for the given display handle.
///
/// This function checks the underlying window system (Wayland, X11, Windows, etc.)
/// and returns the corresponding Vulkan extension names required to create a surface
/// for that system.
pub fn get_required_instance_extensions(display_handle: &impl HasDisplayHandle) -> Vec<*const i8> {
    let mut extensions = vec![b"VK_KHR_surface\0".as_ptr() as *const i8];

    match display_handle.display_handle().map(|h| h.as_raw()) {
        Ok(RawDisplayHandle::Wayland(_)) => {
            extensions.push(b"VK_KHR_wayland_surface\0".as_ptr() as *const i8)
        }
        Ok(RawDisplayHandle::Xlib(_)) => {
            extensions.push(b"VK_KHR_xlib_surface\0".as_ptr() as *const i8)
        }
        Ok(RawDisplayHandle::Xcb(_)) => {
            extensions.push(b"VK_KHR_xcb_surface\0".as_ptr() as *const i8)
        }
        Ok(RawDisplayHandle::Windows(_)) => {
            extensions.push(b"VK_KHR_win32_surface\0".as_ptr() as *const i8)
        }
        _ => {}
    }

    extensions
}

/// Represents an application window.
/// It handles creating the window and handling the received event.
#[derive(Default)]
pub struct ApplicationWindow {
    pub renderer: Option<Renderer>,
    pub instance: Option<Instance>,
    pub window: Option<Window>,
}

impl ApplicationHandler for ApplicationWindow {
    /// Initializes the application window.
    ///
    /// This method is called when the application is resumed. It is the first point
    /// where we are guaranteed to have a valid window handle, which is required to
    /// initialize the Vulkan instance and renderer.
    ///
    /// # Arguments
    /// * `event_loop` - The active event loop.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = event_loop
                .create_window(WindowAttributes::default().with_title(WINDOW_TITLE.to_string()))
                .unwrap();

            let instance = Instance::new(event_loop, &window);
            let renderer = Renderer::new(&instance, &window);

            self.instance = Some(instance);
            self.renderer = Some(renderer);
            self.window = Some(window);

            println!("Window Application created");
        }
    }

    /// Handles window events.
    ///
    /// # Arguments
    /// * `event_loop` - The active event loop.
    /// * `_window_id` - The ID of the window.
    /// * `event` - The received window event.
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    // Window is minimized, skip resizing.
                    return;
                }

                if let (Some(renderer), Some(instance), Some(window)) = (
                    self.renderer.as_mut(),
                    self.instance.as_ref(),
                    self.window.as_ref(),
                ) {
                    renderer.resize(instance, window);
                }
            }

            WindowEvent::RedrawRequested => {
                if let (Some(renderer), Some(instance), Some(window)) = (
                    self.renderer.as_mut(),
                    self.instance.as_ref(),
                    self.window.as_ref(),
                ) {
                    window.request_redraw();
                    renderer.draw_frame(instance, window);
                }
            }

            _ => (),
        }
    }
}
