use kast_event::{Event, WindowEvent, WindowEventPayload};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent as WinitWindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId as WinitWindowId;

use crate::{EventLoopHandler, WindowManager};

pub struct WinitApp<H> {
    manager: WindowManager,
    handler: H,
}

impl<H> WinitApp<H> {
    pub fn new(manager: WindowManager, handler: H) -> Self {
        Self { manager, handler }
    }
}

impl<H: EventLoopHandler> ApplicationHandler for WinitApp<H> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.manager.process_pending_windows(event_loop);
        self.handler.on_resume(&mut self.manager);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WinitWindowId,
        event: WinitWindowEvent,
    ) {
        if let Some(&engine_id) = self.manager.id_map.get(&window_id) {
            match event {
                WinitWindowEvent::CloseRequested => {
                    self.handler.on_event(
                        Event::Window(WindowEvent {
                            window_id: engine_id,
                            payload: WindowEventPayload::CloseRequested,
                        }),
                        &mut self.manager,
                    );
                }
                WinitWindowEvent::RedrawRequested => {
                    self.handler.on_render(&mut self.manager);
                }
                WinitWindowEvent::Resized(size) => {
                    if let Some(window) = self.manager.windows.get_mut(&engine_id) {
                        window.config.size.width = size.width;
                        window.config.size.height = size.height;
                    }

                    self.handler.on_event(
                        Event::Window(WindowEvent {
                            window_id: engine_id,
                            payload: WindowEventPayload::Resized(size.width, size.height),
                        }),
                        &mut self.manager,
                    );
                }
                _ => {
                    // Convert to kast_event::Event and pass to handler
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.manager.process_pending_windows(event_loop);

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        self.handler.on_update(&mut self.manager);

        for window in self.manager.windows.values() {
            if let Some(window) = &window.inner {
                window.request_redraw();
            }
        }

        if self.handler.should_exit() {
            event_loop.exit();
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.handler.on_suspend(&mut self.manager);
    }
}
