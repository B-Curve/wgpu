use crate::objects::block::Block;
use crate::objects::block_material::BlockMaterial;

impl Block {
    
    pub const Air: Block = Block {
        name: "air",
        id: 0,
        material: BlockMaterial::Transparent,
        uv: [[0;2]; 6],
        scale: [0.0, 0.0, 0.0],
        opacity: 0.0,
    };

    pub const Grass: Block = Block {
        name: "grass",
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
        scale: [1.0, 1.0, 1.0],
        opacity: 1.0,
    };

    pub const Dirt: Block = Block {
        name: "dirt",
        id: 2,
        material: BlockMaterial::Solid,
        uv: [[2, 15]; 6],
        scale: [1.0, 1.0, 1.0],
        opacity: 1.0,
    };

    pub const Stone: Block = Block {
        name: "stone",
        id: 3,
        material: BlockMaterial::Solid,
        uv: [[3, 15]; 6],
        scale: [1.0, 1.0, 1.0],
        opacity: 1.0,
    };

    pub const Water: Block = Block {
        name: "water",
        id: 4,
        material: BlockMaterial::Transparent,
        uv: [[4, 15]; 6],
        scale: [1.0, 0.9, 1.0],
        opacity: 0.6,
    };

    pub fn block(id: u8) -> Block {
        match id {
            0 => Self::Air,
            1 => Self::Grass,
            2 => Self::Dirt,
            3 => Self::Stone,
            4 => Self::Water,
            _ => Self::Air,
        }
    }
    
}