use crate::objects::block::Block;
use crate::objects::block_material::BlockMaterial;

impl Block {
    
    pub const Air: Block = Block {
        id: 0,
        material: BlockMaterial::Transparent,
        uv: [[0;2]; 6],
    };

    pub const Grass: Block = Block {
        id: 1,
        material: BlockMaterial::Solid,
        uv: [
            [0, 15],
            [0, 15],
            [0, 15],
            [0, 15],
            [1, 15],
            [2, 15],
        ],
    };

    pub const Dirt: Block = Block {
        id: 2,
        material: BlockMaterial::Solid,
        uv: [[2, 15]; 6],
    };

    pub const Stone: Block = Block {
        id: 3,
        material: BlockMaterial::Solid,
        uv: [[3, 15]; 6],
    };
    
}