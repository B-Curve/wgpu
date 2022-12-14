use crate::objects::block::Block;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TargetVertex {
    pub position: [f32; 3],
}

impl TargetVertex {

    const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
        0 => Float32x3,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TargetVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn load() -> (Vec<TargetVertex>, Vec<u32>) {
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut i_off = 0;
        let p = &Block::POSITIONS;
        let ind = &Block::INDICES;

        for i in 0..6 {
            for v in 0..4 {
                vertices.push(TargetVertex {
                    position: [
                        p[i][v][0],
                        p[i][v][1],
                        p[i][v][2],
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