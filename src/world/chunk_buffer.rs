use wgpu::{Buffer, Device};
use wgpu::util::DeviceExt;
use crate::world::chunk_mesh::ChunkMesh;

pub struct ChunkBuffer {
    pub vertex_buffer: Buffer,
    pub vertex_count: u32,
    pub index_buffer: Buffer,
    pub index_count: u32,

    pub alpha_vertex_buffer: Buffer,
    pub alpha_vertex_count: u32,
    pub alpha_index_buffer: Buffer,
    pub alpha_index_count: u32,
}

impl ChunkBuffer {

    pub fn new(device: &Device, mesh: &ChunkMesh) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let alpha_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.alpha_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let alpha_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.alpha_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            vertex_count: mesh.vertices.len() as u32,
            index_buffer,
            index_count: mesh.indices.len() as u32,

            alpha_vertex_buffer,
            alpha_vertex_count: mesh.alpha_vertices.len() as u32,
            alpha_index_buffer,
            alpha_index_count: mesh.alpha_indices.len() as u32,
        }
    }

}