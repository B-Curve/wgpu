use wgpu::{BindGroup, Buffer, CompareFunction, Device, Queue, RenderPipeline, SurfaceConfiguration, TextureFormat};
use wgpu::util::DeviceExt;
use crate::engine::texture::Texture;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct HotbarVertex {
    position: [f32; 3],
    uv: [f32; 2],
}

pub struct HotbarPipeline {
    pipeline: RenderPipeline,

    vertex_buffer: Buffer,

    diffuse_bind_group: BindGroup,
    diffuse_texture: Texture,
}

impl HotbarPipeline {

    pub fn new(
        device: &Device,
        queue: &Queue,
        config: &SurfaceConfiguration,
    ) -> Self {
        let diffuse_image = image::io::Reader::open("assets/textures/hotbar.png")
            .unwrap()
            .decode()
            .unwrap()
            .flipv();

        let (width, height) = (diffuse_image.width() as f32, diffuse_image.height() as f32);
        let y_scale = height / width;

        let diffuse_texture = Texture::from_image(device, queue, &diffuse_image, Some("hotbar")).unwrap();

        let verts = vec![
            HotbarVertex { position: [-0.25, -1.0, 0.0], uv: [0.0, 0.5] },
            HotbarVertex { position: [0.25, -1.0, 0.0], uv: [1.0, 0.5] },
            HotbarVertex { position: [0.25, -1.0 + y_scale * 0.5, 0.0], uv: [1.0, 1.0] },

            HotbarVertex { position: [-0.25, -1.0, 0.0], uv: [0.0, 0.5] },
            HotbarVertex { position: [0.25, -1.0 + y_scale * 0.5, 0.0], uv: [1.0, 1.0] },
            HotbarVertex { position: [-0.25, -1.0 + y_scale * 0.5, 0.0], uv: [0.0, 1.0] },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hotbar Vertex Buffer"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("hotbar_texture_bind_group_layout"),
        });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler()),
                },
            ],
            label: Some("hotbar_diffuse_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Hotbar Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/hotbar.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Hotbar Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Hotbar Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<HotbarVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x2,
                            }
                        ],
                    }
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: true,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            pipeline,

            vertex_buffer,

            diffuse_bind_group,
            diffuse_texture,
        }
    }

}

pub trait DrawBlock<'a> {
    fn draw_hotbar(
        &mut self,
        pipeline: &'a HotbarPipeline,
    );
}

impl<'a, 'b> DrawBlock<'b> for wgpu::RenderPass<'a>
    where 'b: 'a {
    fn draw_hotbar(
        &mut self,
        pipeline: &'a HotbarPipeline,
    ) {
        self.set_pipeline(&pipeline.pipeline);
        self.set_bind_group(0, &pipeline.diffuse_bind_group, &[]);
        self.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
        self.draw(0..6, 0..1);
    }
}