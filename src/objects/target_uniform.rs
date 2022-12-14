use cgmath::vec3;
use crate::objects::block_face::BlockFace;
use crate::objects::target::Target;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TargetUniform {
    pub position: [f32; 3],
    pub face: u32,
}

impl TargetUniform {

    pub fn new() -> Self {
        Self {
            position: [-1.0;3],
            face: 0,
        }
    }

    pub fn update(&mut self, target: Option<&Target>) {
        let optional_target = Target {
            position: vec3(-1.0, -1.0, -1.0),
            face: BlockFace::None,
            name: String::new(),
        };

        let target = target.unwrap_or(&optional_target);

        self.position = target.position.into();
        self.face = target.face as u32;
    }

}