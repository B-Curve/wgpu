use std::time::Duration;
use cgmath::{Deg, vec3};
use indoc::indoc;
use wgpu::util::StagingBelt;
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, Section, Text};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyboardInput, MouseButton, WindowEvent};
use winit::window::Window;
use crate::scene::camera::Camera;
use crate::scene::camera_uniform::CameraUniform;
use crate::scene::projection::Projection;
use crate::engine::block_pipeline;
use crate::engine::block_pipeline::BlockPipeline;
use crate::engine::block_target_pipeline::{BlockTargetPipeline};
use crate::engine::hotbar_pipeline::{DrawBlock, HotbarPipeline};
use crate::engine::texture::Texture;
use crate::objects::block_face::BlockFace;
use crate::objects::target::Target;
use crate::objects::target_uniform::TargetUniform;
use crate::scene::frustum::Frustum;
use crate::world::world::World;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    staging_belt: StagingBelt,
    glyph_brush: GlyphBrush<()>,

    depth_texture: Texture,

    camera: Camera,
    camera_uniform: CameraUniform,
    projection: Projection,
    frustum: Frustum,

    target_uniform: TargetUniform,
    target: Option<Target>,

    world: World,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            features: wgpu::Features::default() | wgpu::Features::POLYGON_MODE_LINE | wgpu::Features::DEPTH_CLIP_CONTROL | wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER,
            limits: wgpu::Limits::default(),
            label: None,
        }, None).await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        let camera = Camera::new(vec3(0.0, 70.0, 0.0), Deg(0.0), Deg(0.0));
        let camera_uniform = CameraUniform::new();

        let (width, height) = (config.width, config.height);

        let projection = Projection::new(width, height, Deg(90.0), 0.1, 1000.0);

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");
        
        let target_uniform = TargetUniform::new();

        let mut world = World::new(12);
        world.generate(&camera);

        let frustum = Frustum::new(&camera, &projection);

        let staging_belt = StagingBelt::new(1024);

        let font = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!("../../assets/fonts/YatraOne-Regular.ttf"))
            .unwrap();

        let mut glyph_brush = GlyphBrushBuilder::using_font(font)
            .build(&device, config.format);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            staging_belt,
            glyph_brush,
            depth_texture,
            camera,
            camera_uniform,
            frustum,
            target: None,
            target_uniform,
            projection,
            world,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        let (nw, nh) = (new_size.width, new_size.height);

        self.projection.resize(nw, nh);

        if nw > 0 && nh > 0 {
            self.size = new_size;
            self.config.width = nw;
            self.config.height = nh;
            self.surface.configure(&self.device, &self.config);
        }

        self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
    }

    pub fn handle_keyboard_input(&mut self, input: &KeyboardInput) {
        if let Some(code) = input.virtual_keycode {
            self.camera.process_key_input(code, input.state);
        }
    }

    pub fn handle_mouse_input(&mut self, button: &MouseButton, state: &ElementState) {
        match *button {
            MouseButton::Right => if *state == ElementState::Pressed {
                self.world.place_block(self.target.as_ref());
            },
            MouseButton::Left => if *state == ElementState::Pressed {
                self.world.remove_block(self.target.as_ref());
            },
            _ => {},
        }
    }

    pub fn handle_cursor_move(&mut self, position: &(f64, f64)) {
        self.camera.process_mouse_motion(position.0, position.1);
    }

    pub fn update(
        &mut self,
        dt: Duration,
        pipeline: &mut BlockPipeline,
        target_pipeline: &mut BlockTargetPipeline,
        hotbar_pipeline: &mut HotbarPipeline,
    ) {
        self.world.update(&self.device, &self.camera);

        self.target = self.world.get_target(&self.camera);
        self.target_uniform.update(self.target.as_ref());

        self.camera.update(dt);
        self.camera_uniform.update(&self.camera, &self.projection);
        self.frustum.update(&self.camera, &self.projection);

        pipeline.update(&self.queue, &self.camera_uniform);
        target_pipeline.update(&self.queue, &self.camera_uniform, &self.target_uniform);
    }

    pub fn render(
        &mut self,
        block_pipeline: &BlockPipeline,
        target_pipeline: &BlockTargetPipeline,
        hotbar_pipeline: &HotbarPipeline,
        fps: u32,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.4,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            let buffers = self.world.buffers(&self.frustum);

            {
                use crate::engine::block_pipeline::DrawBlock;
                render_pass.attach_pipeline(block_pipeline);
                buffers
                    .iter()
                    .for_each(|b| {
                        render_pass.draw_mesh(b);
                    });
            }

            {
                use crate::engine::block_target_pipeline::DrawBlock;
                render_pass.draw_mesh(target_pipeline);
            }

            {
                use crate::engine::block_pipeline::DrawBlock;
                render_pass.attach_pipeline(block_pipeline);
                buffers
                    .iter()
                    .for_each(|b| {
                        render_pass.draw_alpha_mesh(b);
                    });
            }

            {
                use crate::engine::hotbar_pipeline::DrawBlock;
                render_pass.draw_hotbar(hotbar_pipeline);
            }
        }

        let p = self.camera.position();

        let (w, h) = (self.config.width as f32, self.config.height as f32);

        let target_info = if let Some(target) = &self.target {
            format!(indoc! {"
                Targeted Block: {} [{:?}]
            "}, target.name, target.face)
        } else {
            String::new()
        };

        self.glyph_brush.queue(Section {
            screen_position: (5.0, 0.0),
            bounds: (w, h),
            text: vec![
                Text::new(&format!(
                    indoc! {"
                        FPS: {}
                        Position: [{:.2}, {:.2}, {:.2}]
                        {}
                    "}, fps, p.x, p.y, p.z, target_info)
                ).with_scale(40.0).with_color([1.0, 1.0, 1.0, 1.0])
            ],
            ..Section::default()
        });

        self.glyph_brush.queue(Section {
            screen_position: (w / 2.0 - 30.0, h / 2.0 - 30.0),
            bounds: (w, h),
            text: vec![Text::new("+").with_scale(60.0).with_color([1.0, 1.0, 1.0, 1.0])],
            ..Section::default()
        });

        self.glyph_brush.draw_queued(
            &self.device,
            &mut self.staging_belt,
            &mut encoder,
            &view,
            self.config.width,
            self.config.height,
        ).unwrap();

        self.staging_belt.finish();

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.staging_belt.recall();

        Ok(())
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size.clone()
    }

    pub fn device(&self) -> &wgpu::Device { &self.device }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration { &self.config }

    pub fn queue(&self) -> &wgpu::Queue { &self.queue }

    pub fn camera_unfirom(&self) -> &CameraUniform {
        &self.camera_uniform
    }

    pub fn target_uniform(&self) -> &TargetUniform {
        &self.target_uniform
    }
}