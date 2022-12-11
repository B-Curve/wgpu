use crate::mesh::vertex::Vertex;
use crate::objects::block_material::BlockMaterial;

#[derive(Debug, Copy, Clone)]
pub struct Block {
    pub id: u8,
    pub material: BlockMaterial,
    pub uv: [[u8; 2]; 6],
}

impl Block {

    const POSITIONS: [[[f32; 3]; 4]; 6] = [
        [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]], // Front
        [[1.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [1.0, 1.0, 1.0]], // Back
        [[0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 1.0]], // Left
        [[1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0]], // Right
        [[0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]], // Top
        [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]], // Bottom
    ];

    const NORMALS: [[f32; 3]; 6] = [
        [ 0.0,  0.0, -1.0],
        [ 0.0,  0.0,  1.0],
        [-1.0,  0.0,  0.0],
        [ 1.0,  0.0,  0.0],
        [ 0.0,  1.0,  0.0],
        [ 0.0, -1.0,  0.0],
    ];

    const UV: [[[f32; 2]; 4]; 6] = [
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
    ];

    const INDICES: [[u32; 6]; 6] = [
        [0, 1, 2, 0, 2, 3],
        [0, 1, 2, 0, 2, 3],
        [0, 1, 2, 0, 2, 3],
        [0, 1, 2, 0, 2, 3],
        [0, 1, 2, 0, 2, 3],
        [0, 1, 2, 0, 2, 3],
    ];

    pub fn build_faces(
        &self,
        faces: [bool; 6],
    ) -> (Vec<Vertex>, Vec<u32>) {

        let mut vertices = vec![];
        let mut indices = vec![];
        let p = &Self::POSITIONS;
        let u = &Self::UV;
        let ind = &Self::INDICES;
        let mut i_off = 0;

        let ux = 16.0 / 256.0;
        let uy = 16.0 / 256.0;
        let uvi = self.uv;

        for i in 0..6 {
            if !faces[i] { continue; }

            for v in 0..4 {
                vertices.push(Vertex {
                    position: [p[i][v][0], p[i][v][1], p[i][v][2]],
                    uv: [
                        u[i][v][0] * ux + ux * uvi[i][0] as f32,
                        u[i][v][1] * uy + uy * uvi[i][1] as f32,
                    ],
                });
            }

            indices.extend_from_slice(&[
                ind[i][0] + i_off, ind[i][1] + i_off, ind[i][2] + i_off,
                ind[i][3] + i_off, ind[i][4] + i_off, ind[i][5] + i_off,
            ]);

            i_off += 4;
        }

        (vertices, indices)
    }

}