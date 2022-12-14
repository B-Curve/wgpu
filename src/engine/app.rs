use std::time::Duration;
use cgmath::{Deg, Rad, vec3};
use crossbeam::channel::Sender;
use winit::event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::window::Window;
use crate::{EventLoopRequest, State};
use crate::scene::camera::Camera;
use crate::scene::camera_uniform::CameraUniform;
use crate::scene::projection::Projection;
use crate::engine::block_pipeline::BlockPipeline;
use crate::engine::block_target_pipeline::BlockTargetPipeline;
use crate::engine::hotbar_pipeline::HotbarPipeline;

pub struct App {
    state: State,
    block_pipeline: BlockPipeline,
    block_target_pipeline: BlockTargetPipeline,
    hotbar_pipeline: HotbarPipeline,
    event_loop_sender: Sender<EventLoopRequest>,
}

impl App {

    pub async fn new(window: &Window, event_loop_sender: Sender<EventLoopRequest>) -> Self {
        let state = State::new(window)
            .await;

        let block_pipeline = BlockPipeline::new(
            state.device(),
            state.queue(),
            state.config(),
            state.camera_unfirom(),
        );

        let block_target_pipeline = BlockTargetPipeline::new(
            state.device(),
            state.config(),
            state.camera_unfirom(),
            state.target_uniform(),
        );

        let hotbar_pipeline = HotbarPipeline::new(
            state.device(),
            state.queue(),
            state.config(),
        );

        Self {
            state,
            block_pipeline,
            block_target_pipeline,
            hotbar_pipeline,
            event_loop_sender,
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => self.state.handle_cursor_move(delta),
            _ => {},
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                match input {
                    &KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), state: ElementState::Pressed, .. } => {
                        self.event_loop_sender.send(EventLoopRequest::Close).unwrap();
                    },
                    input => self.state.handle_keyboard_input(input),
                }
            },
            WindowEvent::MouseInput { button, state: e_state, .. } => self.state.handle_mouse_input(button, e_state),
            WindowEvent::CloseRequested => self.event_loop_sender.send(EventLoopRequest::Close).unwrap(),
            WindowEvent::Resized(size) => self.state.resize(*size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => self.state.resize(**new_inner_size),
            _ => {},
        }
    }

    pub fn handle_redraw_request(&mut self, dt: Duration, fps: u32) {
        self.state.update(
            dt,
            &mut self.block_pipeline,
            &mut self.block_target_pipeline,
            &mut self.hotbar_pipeline,
        );

        match self.state.render(
            &self.block_pipeline,
            &self.block_target_pipeline,
            &self.hotbar_pipeline,
            fps,
        ) {
            Ok(_) => {},
            Err(wgpu::SurfaceError::Lost) => self.state.resize(self.state.size()),
            Err(wgpu::SurfaceError::OutOfMemory) => self.event_loop_sender.send(EventLoopRequest::Close).unwrap(),
            Err(e) => eprintln!("{:?}", e),
        }
    }

}