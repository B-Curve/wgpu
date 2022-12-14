use cgmath::Vector3;
use crate::objects::block_face::BlockFace;

#[derive(Debug, Clone)]
pub struct Target {
    pub position: Vector3<f32>,
    pub face: BlockFace,
    pub name: String,
}