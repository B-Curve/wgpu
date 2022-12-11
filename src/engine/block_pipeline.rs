use wgpu::{BindGroup, Buffer, CompareFunction, Device, Queue, RenderPass, RenderPipeline, SurfaceConfiguration, TextureFormat};
use wgpu::util::DeviceExt;
use crate::engine::texture::Texture;
use crate::mesh::vertex::Vertex;
use crate::objects::block::Block;
use crate::scene::camera_uniform::CameraUniform;

pub struct BlockPipeline {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,

    diffuse_bind_group: BindGroup,
    diffuse_texture: Texture,

    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
}

impl BlockPipeline {

    pub fn new(
        device: &Device,
        queue: &Queue,
        config: &SurfaceConfiguration,
        camera_uniform: &CameraUniform,
    ) -> Self {
        let block = Block::Dirt;
        let (vb, ib) = block.build_faces([true, true, true, true, true, true]);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Vertex Buffer"),
            contents: bytemuck::cast_slice(&vb),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Index Buffer"),
            contents: bytemuck::cast_slice(&ib),
            usage: wgpu::BufferUsages::INDEX,
        });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Camera Buffer"),
            contents: bytemuck::cast_slice(&[*camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let diffuse_image = image::io::Reader::open("assets/textures/atlas.png")
            .unwrap()
            .decode()
            .unwrap()
            .flipv();

        println!("{:?}", vb);

        let diffuse_texture = Texture::from_image(device, queue, &diffuse_image, Some("atlas")).unwrap();

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
            label: Some("block_texture_bind_group_layout"),
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
            label: Some("block_diffuse_bind_group"),
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("block_camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("block_camera_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Block Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Block Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Block Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Front),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
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
            index_buffer,
            index_count: ib.len() as u32,

            diffuse_bind_group,
            diffuse_texture,

            camera_buffer,
            camera_bind_group,
        }
    }

    pub fn update(
        &mut self,
        queue: &Queue,
        camera_uniform: &CameraUniform,
    ) {
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[*camera_uniform]));
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

}

pub trait DrawBlock<'a> {
    fn draw_mesh(
        &mut self,
        pipeline: &'a BlockPipeline,
    );
}

impl<'a, 'b> DrawBlock<'b> for wgpu::RenderPass<'a>
where 'b: 'a {
    fn draw_mesh(&mut self, pipeline: &'a BlockPipeline) {
        self.set_pipeline(pipeline.pipeline());

        self.set_bind_group(0, &pipeline.camera_bind_group, &[]);
        self.set_bind_group(1, &pipeline.diffuse_bind_group, &[]);

        self.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
        self.set_index_buffer(pipeline.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..pipeline.index_count, 0, 0..1);
    }
}