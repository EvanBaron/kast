use crate::window::ApplicationWindow;
use winit::event_loop::EventLoop;

mod graphics;
mod window;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut application_window = ApplicationWindow::default();
    let _ = event_loop.run_app(&mut application_window).unwrap();
}
