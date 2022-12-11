use cgmath::vec4;
use crate::scene::camera::Camera;
use crate::scene::projection::Projection;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    position: [f32; 4],
    projection: [[f32; 4]; 4],
}

impl CameraUniform {

    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            position: [0.0; 4],
            projection: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, camera: &Camera, projection: &Projection) {
        let p = camera.position();
        self.position = vec4(p.x, p.y, p.z, 1.0).into();
        self.projection = (projection.calculate_matrix() * camera.calculate_matrix()).into();
    }

}