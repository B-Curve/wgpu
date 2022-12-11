mod window;
mod engine;
mod mesh;
mod objects;
mod scene;
mod world;

use crossbeam::channel::unbounded;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{CursorGrabMode, WindowBuilder};
use crate::engine::app::App;
use crate::window::event_loop_request::EventLoopRequest;
use crate::window::state::State;

async fn run() {
    // env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)
        .unwrap();

    window.set_cursor_grab(CursorGrabMode::Confined);
    window.set_cursor_visible(false);
    window.set_inner_size(LogicalSize::new(1920, 1080));

    if let Some(monitor) = window.current_monitor() {
        let size = monitor.size();
        let wsize = window.outer_size();

        let off_x = (size.width - wsize.width) / 2;
        let off_y = (size.height - wsize.height) / 2;

        window.set_outer_position(LogicalPosition::new(off_x, off_y));
    }

    let (sender, receiver) = unbounded::<EventLoopRequest>();

    let mut app = App::new(&window, sender).await;

    let mut last_render_time = instant::Instant::now();
    let mut last_fps_check = instant::Instant::now();
    let mut fps = 0;

    event_loop.run(move |event, _, control_flow| match event {
        Event::DeviceEvent { event, .. } => app.handle_device_event(&event),
        Event::WindowEvent {
            ref event,
            window_id,
        } if window.id() == window_id => app.handle_window_event(event),
        Event::RedrawRequested(id) if window.id() == id => {
            let now = instant::Instant::now();
            let dt = now - last_render_time;

            if now - last_fps_check >= instant::Duration::from_secs(2) {
                fps = (1.0 / dt.as_secs_f32()).round() as u32;
                last_fps_check = now;
            }

            last_render_time = now;

            app.handle_redraw_request(dt);
        },
        Event::MainEventsCleared => {
            window.request_redraw();

            match receiver.try_recv() {
                Ok(EventLoopRequest::Close) => *control_flow = ControlFlow::Exit,
                _ => {},
            }
        },
        _ => {},
    });
}

fn main() {
    pollster::block_on(run());
}
