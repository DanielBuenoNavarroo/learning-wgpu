use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

use crate::renderer::renderer::Renderer;

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    last_frame_time: std::time::Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            last_frame_time: std::time::Instant::now(),
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self).unwrap();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title("My Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));

        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        let renderer = Renderer::new(window.clone());

        self.window = Some(window);
        self.renderer = Some(renderer);

        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height);
                }
                window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    let now = std::time::Instant::now();
                    let dt = now - self.last_frame_time;
                    self.last_frame_time = now;

                    renderer.render(dt);
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(renderer) = self.renderer.as_mut() {
                    let pressed = event.state == ElementState::Pressed;

                    if let PhysicalKey::Code(keycode) = event.physical_key {
                        renderer.handle_key(event_loop, keycode, pressed);
                    }
                }
            }

            _ => {}
        }
    }
}
