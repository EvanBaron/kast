use crate::window::ApplicationWindow;
use winit::event_loop::EventLoop;

mod graphics;
mod window;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut application_window = ApplicationWindow::default();
    let _ = event_loop.run_app(&mut application_window).unwrap();
}
