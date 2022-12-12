use crate::mesh::vertex::Vertex;

pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,

    pub alpha_vertices: Vec<Vertex>,
    pub alpha_indices: Vec<u32>,
}