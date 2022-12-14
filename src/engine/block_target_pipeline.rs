use wgpu::{BindGroup, Buffer, CompareFunction, Device, Queue, RenderPass, RenderPipeline, SurfaceConfiguration, TextureFormat};
use wgpu::util::DeviceExt;
use crate::engine::texture::Texture;
use crate::mesh::target_vertex::TargetVertex;
use crate::mesh::vertex::Vertex;
use crate::objects::block::Block;
use crate::objects::target::Target;
use crate::objects::target_uniform::TargetUniform;
use crate::scene::camera_uniform::CameraUniform;
use crate::world::chunk::Chunk;
use crate::world::chunk_buffer::ChunkBuffer;
use crate::world::world::World;

pub struct BlockTargetPipeline {
    pipeline: RenderPipeline,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,

    target_buffer: Buffer,
    target_bind_group: BindGroup,

    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
}

impl BlockTargetPipeline {

    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        camera_uniform: &CameraUniform,
        target_uniform: &TargetUniform,
    ) -> Self {
        let (verts, inds) = TargetVertex::load();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Target Vertex Buffer"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Target Index Buffer"),
            contents: bytemuck::cast_slice(&inds),
            usage: wgpu::BufferUsages::INDEX,
        });

        let target_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Target Camera Buffer"),
            contents: bytemuck::cast_slice(&[*target_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let target_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("target_camera_bind_group_layout"),
        });

        let target_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &target_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: target_buffer.as_entire_binding(),
            }],
            label: Some("target_camera_bind_group"),
        });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Target Buffer"),
            contents: bytemuck::cast_slice(&[*camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
            label: Some("block_target_camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("block_target_camera_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Block Target Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/block-target.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Block Target Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &target_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Block Target Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    TargetVertex::desc(),
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
                polygon_mode: wgpu::PolygonMode::Line,
                unclipped_depth: true,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
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
            index_count: inds.len() as u32,

            target_buffer,
            target_bind_group,

            camera_buffer,
            camera_bind_group,
        }
    }

    pub fn update(
        &mut self,
        queue: &Queue,
        camera_uniform: &CameraUniform,
        target_uniform: &TargetUniform,
    ) {
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[*camera_uniform]));
        queue.write_buffer(&self.target_buffer, 0, bytemuck::cast_slice(&[*target_uniform]));
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

}

pub trait DrawBlock<'a> {
    fn draw_mesh(
        &mut self,
        pipeline: &'a BlockTargetPipeline,
    );
}

impl<'a, 'b> DrawBlock<'b> for wgpu::RenderPass<'a>
    where 'b: 'a {
    fn draw_mesh(
        &mut self,
        pipeline: &'a BlockTargetPipeline
    ) {
        self.set_pipeline(pipeline.pipeline());

        self.set_bind_group(0, &pipeline.camera_bind_group, &[]);
        self.set_bind_group(1, &pipeline.target_bind_group, &[]);

        self.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
        self.set_index_buffer(pipeline.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..pipeline.index_count, 0, 0..1);
    }
}