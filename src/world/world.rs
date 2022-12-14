use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;
use cgmath::{EuclideanSpace, frustum, MetricSpace, Point3, vec3, Vector3};
use collision::{Aabb3, Continuous, Ray, Relation};
use noise::{Fbm, MultiFractal, Perlin};
use threadpool::ThreadPool;
use wgpu::Device;
use crate::objects::block::Block;
use crate::objects::block_face::BlockFace;
use crate::objects::target::Target;
use crate::scene::camera::Camera;
use crate::scene::frustum::Frustum;
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
            pool: ThreadPool::new(16),
            render_distance,
            noise,
            buffers: HashMap::new(),
            vertex_count: 0,
        }
    }

    pub fn generate(&mut self, camera: &Camera) {
        let (x, z) = Self::to_local_position(camera.position());
        let r = self.render_distance;

        for x in (x - (r + 1))..(x + (r + 1)) {
            for z in (z - (r + 1))..(z + (r + 1)) {
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

        for x in (x - (r + 1))..(x + (r + 1)) {
            for z in (z - (r + 1))..(z + (r + 1)) {
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
                    if chunk.borrow().needs_buffer() {
                        let buffer = ChunkBuffer::new(device, chunk.borrow().mesh());
                        self.vertex_count += buffer.vertex_count;
                        next_buffers.insert((x, z), buffer);
                        chunk.borrow_mut().set_needs_buffer(false);
                    } else {
                        self.vertex_count += b.vertex_count;
                        next_buffers.insert((x, z), b);
                    }
                } else if chunk.borrow().has_mesh() {
                    let buffer = ChunkBuffer::new(device, chunk.borrow().mesh());
                    self.vertex_count += buffer.vertex_count;
                    next_buffers.insert((x, z), buffer);
                    chunk.borrow_mut().set_needs_buffer(false);
                }
            }
        }

        self.buffers = next_buffers;
    }

    pub fn get_target(&self, camera: &Camera) -> Option<Target> {
        let ray = Ray::new(Point3::from_vec(camera.position().clone()), camera.front().clone());

        let (cx, cy, cz) = (
            camera.position().x.floor() as i32,
            camera.position().y.floor() as i32,
            camera.position().z.floor() as i32,
        );

        let mut nearest_block: Option<Target> = None;
        let mut nearest = 1000f32;
        let mut nearest_block_d = Block::Air;
        let mut nearest_pos = (0, 0, 0);

        for x in cx - 6..cx + 6 {
            for y in cy - 6..cy + 6 {
                for z in cz - 6..cz + 6 {
                    if (x as f32 - camera.position().x).abs() > 6.0
                        || (y as f32 - camera.position().y).abs() > 6.0
                        || (z as f32 - camera.position().z).abs() > 6.0 {
                        continue;
                    }

                    let block_id = self.get_block(x, y, z)
                        .unwrap_or(0);

                    if block_id == 0 || block_id == Block::Water.id { continue; }

                    let bb = Aabb3::new(
                        Point3::new(x as f32, y as f32, z as f32),
                        Point3::new(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0),
                    );

                    if let Some(point) = bb.intersection(&ray) {
                        let distance = point.distance(Point3::from_vec(camera.position() + camera.front()));

                        if distance < nearest {
                            nearest = distance;
                            nearest_block_d = Block::block(block_id);
                            nearest_pos = (x, y, z);
                            nearest_block = Some(Target {
                                position: vec3(x as f32, y as f32, z as f32),
                                face: if (point.x - x as f32).abs() < 0.0001 {
                                    BlockFace::Front
                                } else if (point.x - (x as f32 + 1.0)).abs() < 0.0001 {
                                    BlockFace::Back
                                } else if (point.z - z as f32).abs() < 0.0001 {
                                    BlockFace::Left
                                } else if (point.z - (z as f32 + 1.0)).abs() < 0.0001 {
                                    BlockFace::Right
                                } else if (point.y - y as f32).abs() < 0.0001 {
                                    BlockFace::Bottom
                                } else {
                                    BlockFace::Top
                                },
                                name: String::from(Block::block(block_id).name),
                            });
                        }
                    }
                }
            }
        }

        nearest_block
    }

    pub fn place_block(&mut self, target: Option<&Target>) {
        if let Some(target) = target {
            let p = target.position;
            let face = target.face;

            let spot = match face {
                BlockFace::Top => vec3(p.x, p.y + 1.0, p.z),
                BlockFace::Bottom => vec3(p.x, p.y - 1.0, p.z),
                BlockFace::Left => vec3(p.x, p.y, p.z - 1.0),
                BlockFace::Right => vec3(p.x, p.y, p.z + 1.0),
                BlockFace::Front => vec3(p.x - 1.0, p.y, p.z),
                BlockFace::Back => vec3(p.x + 1.0, p.y, p.z),
                _ => return,
            };

            let s = vec3(spot.x.floor() as i32, spot.y.floor() as i32, spot.z.floor() as i32);

            let chunk = self.get_chunk(s.x, s.y, s.z).unwrap();

            let (lx, lz) = {
                let p = chunk.borrow().world_position();
                ((s.x - p.0).abs(), (s.z - p.1).abs())
            };

            let (cx, cz) = chunk.borrow().local_position();

            chunk.borrow_mut().place_block_at_world_position((s.x, s.y, s.z));

            if lx == 0 {
                if let Some(chunk) = self.chunks.get(&(cx - 1, cz)) {
                    chunk.borrow_mut().clear_right();
                }
            } else if lx == Chunk::WIDTH - 1 {
                if let Some(chunk) = self.chunks.get(&(cx + 1, cz)) {
                    chunk.borrow_mut().clear_left();
                }
            } else if lz == 0 {
                if let Some(chunk) = self.chunks.get(&(cx, cz - 1)) {
                    chunk.borrow_mut().clear_back();
                }
            } else if lz == Chunk::DEPTH - 1 {
                if let Some(chunk) = self.chunks.get(&(cx, cz + 1)) {
                    chunk.borrow_mut().clear_front();
                }
            }
        }
    }

    pub fn buffers(&self, frustum: &Frustum) -> Vec<&ChunkBuffer> {
        let f = frustum.get();

        self.buffers
            .iter()
            .filter(|((x, z), _)| {
                let bb = Aabb3::new(
                    Point3::new(*x as f32 * 16.0, 0.0, *z as f32 * 16.0),
                    Point3::new(*x as f32 * 16.0 + 16.0, 256.0, *z as f32 * 16.0 + 16.0),
                );

                f.contains(&bb) != Relation::Out
            })
            .map(|(a, b)| b)
            .collect()
    }

    pub fn to_local_position(position: &Vector3<f32>) -> (i32, i32) {
        let (x, z) = (position.x as i32, position.z as i32);

        (x / Chunk::WIDTH, z / Chunk::DEPTH)
    }

    pub fn get_chunk(&self, x: i32, y: i32, z: i32) -> Option<&RefCell<Chunk>> {
        let (cx, cz) = (
            (x as f32 / Chunk::WIDTH as f32).floor() as i32,
            (z as f32 / Chunk::DEPTH as f32).floor() as i32,
        );

        self.chunks.get(&(cx, cz))
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<u8> {
        let (cx, cz) = (
            (x as f32 / Chunk::WIDTH as f32).floor() as i32,
            (z as f32 / Chunk::DEPTH as f32).floor() as i32,
        );

        if let Some(chunk) = self.chunks.get(&(cx, cz)) {
            chunk.borrow().block_at_world_position((x, y, z))
                .cloned()
        } else {
            None
        }
    }
}