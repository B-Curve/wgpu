use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;
use cgmath::Vector3;
use noise::{Fbm, MultiFractal, Perlin};
use threadpool::ThreadPool;
use wgpu::Device;
use crate::scene::camera::Camera;
use crate::world::chunk::Chunk;
use crate::world::chunk_buffer::ChunkBuffer;

pub struct World {
    chunks: HashMap<(i32, i32), RefCell<Chunk>>,
    pool: ThreadPool,

    render_distance: i32,

    noise: Fbm<Perlin>,

    buffers: HashMap<(i32, i32), ChunkBuffer>,

    vertex_count: u32,
}

impl World {

    pub fn new(render_distance: i32) -> Self {
        let noise = Fbm::new(1)
            .set_octaves(4)
            .set_frequency(0.0348)
            .set_persistence(1.6)
            .set_lacunarity(0.2);

        Self {
            chunks: HashMap::new(),
            pool: ThreadPool::new(8),
            render_distance,
            noise,
            buffers: HashMap::new(),
            vertex_count: 0,
        }
    }

    pub fn generate(&mut self, camera: &Camera) {
        let (x, z) = Self::to_local_position(camera.position());
        let r = self.render_distance;

        for x in (x-(r+1))..(x+(r+1)) {
            for z in (z-(r+1))..(z+(r+1)) {
                let chunk = Chunk::new((x, z), &self.noise);
                self.chunks.insert((x, z), RefCell::new(chunk));
            }
        }

        for ((x, z), chunk) in self.chunks.iter() {
            let left = if let Some(c) = self.chunks.get(&(*x - 1, *z)) {
                c.borrow().blocks().clone()
            } else {
                vec![]
            };

            let right = if let Some(c) = self.chunks.get(&(*x + 1, *z)) {
                c.borrow().blocks().clone()
            } else {
                vec![]
            };

            let front = if let Some(c) = self.chunks.get(&(*x, *z - 1)) {
                c.borrow().blocks().clone()
            } else {
                vec![]
            };

            let back = if let Some(c) = self.chunks.get(&(*x, *z + 1)) {
                c.borrow().blocks().clone()
            } else {
                vec![]
            };

            chunk.borrow_mut().set_left(&left);
            chunk.borrow_mut().set_right(&right);
            chunk.borrow_mut().set_front(&front);
            chunk.borrow_mut().set_back(&back);
        }
    }

    pub fn update(&mut self, device: &Device, camera: &Camera) {
        let (x, z) = Self::to_local_position(camera.position());
        let r = self.render_distance;
        let mut next_buffers = HashMap::new();
        self.vertex_count = 0;

        for x in (x-(r+1))..(x+(r+1)) {
            for z in (z-(r+1))..(z+(r+1)) {
                let chunk = if let Some(chunk) = self.chunks.get(&(x, z)) {
                    chunk
                } else {
                    self.chunks.insert((x, z), RefCell::new(Chunk::new((x, z), &self.noise)));
                    self.chunks.get(&(x, z)).unwrap()
                };

                if chunk.borrow().left().is_none() {
                    if let Some(c) = self.chunks.get(&(x - 1, z)) {
                        chunk.borrow_mut().set_left(c.borrow().blocks());
                    }
                } else if chunk.borrow().right().is_none() {
                    if let Some(c) = self.chunks.get(&(x + 1, z)) {
                        chunk.borrow_mut().set_right(c.borrow().blocks());
                    }
                } else if chunk.borrow().front().is_none() {
                    if let Some(c) = self.chunks.get(&(x, z - 1)) {
                        chunk.borrow_mut().set_front(c.borrow().blocks());
                    }
                } else if chunk.borrow().back().is_none() {
                    if let Some(c) = self.chunks.get(&(x, z + 1)) {
                        chunk.borrow_mut().set_back(c.borrow().blocks());
                    }
                }

                chunk.borrow_mut().update(&mut self.pool);

                if let Some(b) = self.buffers.remove(&(x, z)) {
                    self.vertex_count += b.vertex_count;
                    next_buffers.insert((x, z), b);
                } else if chunk.borrow().has_mesh() {
                    let buffer = ChunkBuffer::new(device, chunk.borrow().mesh());
                    self.vertex_count += buffer.vertex_count;
                    next_buffers.insert((x, z), buffer);
                }
            }
        }

        self.buffers = next_buffers;
    }

    pub fn buffers(&self) -> Vec<&ChunkBuffer> {
        self.buffers
            .iter()
            .map(|(a, b)| b)
            .collect()
    }

    pub fn to_local_position(position: &Vector3<f32>) -> (i32, i32) {
        let (x, z) = (position.x as i32, position.z as i32);

        (x / Chunk::WIDTH, z / Chunk::DEPTH)
    }

}