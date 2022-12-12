use bytemuck::Contiguous;
use crossbeam::channel::{Receiver, Sender};
use noise::{Fbm, NoiseFn, Perlin};
use threadpool::ThreadPool;
use crate::objects::block::Block;
use crate::objects::block_material::BlockMaterial;
use crate::world::chunk_mesh::ChunkMesh;

pub struct Chunk {
    local_position: (i32, i32),
    world_position: (i32, i32),

    blocks: Vec<u8>,
    mesh: ChunkMesh,
    mesh_generated: bool,
    generating_mesh: bool,

    sender: Sender<ChunkMesh>,
    receiver: Receiver<ChunkMesh>,

    left: Option<Vec<u8>>,
    right: Option<Vec<u8>>,
    front: Option<Vec<u8>>,
    back: Option<Vec<u8>>,
}

impl Chunk {
    pub const WIDTH: i32 = 16;
    pub const HEIGHT: i32 = 256;
    pub const DEPTH: i32 = 16;
    pub const SIZE: i32 = Chunk::WIDTH * Chunk::HEIGHT * Chunk::DEPTH;

    pub fn new(local_position: (i32, i32), noise: &Fbm<Perlin>) -> Self {
        let (sender, receiver) = crossbeam::channel::unbounded();

        let mut chunk = Self {
            local_position,
            world_position: Self::local_to_world_position(local_position),
            blocks: vec![Block::Air.id; Chunk::SIZE as usize],
            mesh: ChunkMesh {
                vertices: vec![],
                indices: vec![],
                alpha_vertices: vec![],
                alpha_indices: vec![],
            },
            mesh_generated: false,
            generating_mesh: false,
            sender,
            receiver,
            left: None,
            right: None,
            front: None,
            back: None,
        };

        chunk.generate_blocks(noise);

        chunk
    }

    fn generate_blocks(&mut self, noise: &Fbm<Perlin>) {
        for x in 0..Chunk::WIDTH {
            for z in 0..Chunk::DEPTH {
                let n = ((noise.get([
                    (x + self.world_position.0) as f64 + 0.01,
                    (z + self.world_position.1) as f64 + 0.01
                ]) + 2.0) * 32.0) as i32;

                let chunk_height = n.max(60);

                for y in 0..(chunk_height + 1) {
                    if y > n {
                        self.blocks[Self::xyz_to_index(x, y, z)] = Block::Water.id;
                    } else {
                        self.blocks[Self::xyz_to_index(x, y, z)] = Block::Grass.id;
                    }
                }
            }
        }
    }

    pub fn update(&mut self, pool: &mut ThreadPool) {
        if let Ok(mesh) = self.receiver.try_recv() {
            self.mesh = mesh;
            self.generating_mesh = false;
            self.mesh_generated = true;
            return;
        } else if !self.mesh_generated && !self.generating_mesh {
            self.generate_mesh(pool);
        }
    }

    fn generate_mesh(&mut self, pool: &mut ThreadPool) {
        let (
            left,
            right,
            front,
            back,
        ) = if let (Some(left), Some(right), Some(front), Some(back)) = (
            &self.left,
            &self.right,
            &self.front,
            &self.back,
        ) {
            (left, right, front, back)
        } else {
            return;
        };

        self.generating_mesh = true;
        self.mesh_generated = false;

        let blocks = self.blocks.clone();
        let world_position = self.world_position;
        let sender = self.sender.clone();
        let left = left.clone();
        let right = right.clone();
        let front = front.clone();
        let back = back.clone();

        pool.execute(move || {
            let mut vertices = vec![];
            let mut indices = vec![];
            let mut alpha_vertices = vec![];
            let mut alpha_indices = vec![];
            let mut solid_index_offset = 0;
            let mut alpha_index_offset = 0;

            for (i, b) in blocks.iter().enumerate() {
                let b = *b;
                let block = Block::block(b);

                if Block::Air.id == block.id { continue; }

                let faces = [
                    Self::has_front(&blocks, &front, i),
                    Self::has_back(&blocks, &back, i),
                    Self::has_left(&blocks, &left, i),
                    Self::has_right(&blocks, &right, i),
                    Self::has_top(&blocks, i),
                    Self::has_bottom(&blocks, i),
                ];

                let (x, y, z) = Self::index_to_xyz(i);
                let (x, y, z) = ((x + world_position.0) as f32, y as f32, (z + world_position.1) as f32);

                let (verts, inds) = block.build_faces(
                    x,
                    y,
                    z,
                    faces,
                    if block.material == BlockMaterial::Solid { solid_index_offset } else { alpha_index_offset },
                );

                if block.material == BlockMaterial::Solid {
                    solid_index_offset += verts.len() as u32;
                    vertices.extend_from_slice(verts.as_slice());
                    indices.extend_from_slice(inds.as_slice());
                } else {
                    alpha_index_offset += verts.len() as u32;
                    alpha_vertices.extend_from_slice(verts.as_slice());
                    alpha_indices.extend_from_slice(inds.as_slice());
                }
            }

            sender.send(ChunkMesh {
                vertices,
                indices,

                alpha_vertices,
                alpha_indices,
            });
        });
    }

    pub fn has_left(blocks: &Vec<u8>, left: &Vec<u8>, index: usize) -> bool {
        let (x, y, z) = Self::index_to_xyz(index);
        let block = Block::block(blocks[index]);

        if x == 0 {
            let left = left.get(Self::xyz_to_index(Chunk::WIDTH - 1, y, z))
                .unwrap_or(&Block::Air.id);

            let other = Block::block(*left);

            !Self::is_hidden(&block, &other)
        } else {
            let left = blocks.get(Self::xyz_to_index(x - 1, y, z))
                .unwrap_or(&Block::Air.id);

            let other = Block::block(*left);

            !Self::is_hidden(&block, &other)
        }
    }

    pub fn has_right(blocks: &Vec<u8>, right: &Vec<u8>, index: usize) -> bool {
        let (x, y, z) = Self::index_to_xyz(index);
        let block = Block::block(blocks[index]);

        if x + 1 == Chunk::WIDTH {
            let right = right.get(Self::xyz_to_index(0, y, z))
                .unwrap_or(&Block::Air.id);

            let other = Block::block(*right);

            !Self::is_hidden(&block, &other)
        } else {
            let right = blocks.get(Self::xyz_to_index(x + 1, y, z))
                .unwrap_or(&Block::Air.id);

            let other = Block::block(*right);

            !Self::is_hidden(&block, &other)
        }
    }

    pub fn has_top(blocks: &Vec<u8>, index: usize) -> bool {
        let block = Block::block(blocks[index]);

        let above = blocks.get(index + Chunk::HEIGHT as usize)
            .unwrap_or(&Block::Air.id);

        let other = Block::block(*above);

        !Self::is_hidden(&block, &other)
    }

    pub fn has_bottom(blocks: &Vec<u8>, index: usize) -> bool {
        let index = if index < Chunk::HEIGHT as usize { std::num::NonZeroUsize::MAX_VALUE } else { index };

        let below = blocks.get(index - Chunk::HEIGHT as usize)
            .unwrap_or(&Block::Air.id);

        let other = Block::block(*below);

        if index == std::num::NonZeroUsize::MAX_VALUE {
            true
        } else {
            !Self::is_hidden(&Block::block(blocks[index]), &other)
        }
    }

    pub fn has_front(blocks: &Vec<u8>, front: &Vec<u8>, index: usize) -> bool {
        let (x, y, z) = Self::index_to_xyz(index);
        let block = Block::block(blocks[index]);

        if z == 0 {
            let front = front.get(Self::xyz_to_index(x, y, Self::DEPTH - 1))
                .unwrap_or(&Block::Air.id);

            let other = Block::block(*front);

            !Self::is_hidden(&block, &other)
        } else {
            let front = blocks.get(Self::xyz_to_index(x, y, z - 1))
                .unwrap_or(&Block::Air.id);

            let other = Block::block(*front);

            !Self::is_hidden(&block, &other)
        }
    }

    pub fn has_back(blocks: &Vec<u8>, back: &Vec<u8>, index: usize) -> bool {
        let (x, y, z) = Self::index_to_xyz(index);
        let block = Block::block(blocks[index]);

        if z + 1 == Chunk::DEPTH {
            let back = back.get(Self::xyz_to_index(x, y, 0))
                .unwrap_or(&Block::Air.id);

            let other = Block::block(*back);

            !Self::is_hidden(&block, &other)
        } else {
            let back = blocks.get(Self::xyz_to_index(x, y, z + 1))
                .unwrap_or(&Block::Air.id);

            let other = Block::block(*back);

            !Self::is_hidden(&block, &other)
        }
    }

    fn is_hidden(block: &Block, other: &Block) -> bool {
        if other.material == BlockMaterial::Solid {
            true
        } else if other.id == Block::Water.id && block.id == Block::Water.id {
            true
        } else {
            false
        }
    }

    pub fn xyz_to_index(x: i32, y: i32, z: i32) -> usize {
        (x + z * Chunk::WIDTH + y * Chunk::HEIGHT) as usize
    }

    pub fn index_to_xyz(index: usize) -> (i32, i32, i32) {
        let mut i = index as i32;
        let y = i / Chunk::HEIGHT;
        let yr = i % Chunk::HEIGHT;
        let z = yr / Chunk::WIDTH;
        let zr = yr % Chunk::WIDTH;
        (zr, y, z)
    }

    pub fn local_to_world_position(local_position: (i32, i32)) -> (i32, i32) {
        (local_position.0 * Chunk::WIDTH, local_position.1 * Chunk::DEPTH)
    }

    pub fn blocks(&self) -> &Vec<u8> {
        &self.blocks
    }

    pub fn left(&self) -> &Option<Vec<u8>> {
        &self.left
    }

    pub fn set_left(&mut self, left: &Vec<u8>) {
        self.left = Some(left.clone());
    }

    pub fn right(&self) -> &Option<Vec<u8>> {
        &self.right
    }

    pub fn set_right(&mut self, right: &Vec<u8>) {
        self.right = Some(right.clone());
    }

    pub fn front(&self) -> &Option<Vec<u8>> {
        &self.front
    }

    pub fn set_front(&mut self, front: &Vec<u8>) {
        self.front = Some(front.clone());
    }

    pub fn back(&self) -> &Option<Vec<u8>> {
        &self.back
    }

    pub fn set_back(&mut self, back: &Vec<u8>) {
        self.back = Some(back.clone());
    }

    pub fn mesh(&self) -> &ChunkMesh {
        &self.mesh
    }

    pub fn has_mesh(&self) -> bool {
        self.mesh_generated
    }
}