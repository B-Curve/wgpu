use cgmath::{Angle, InnerSpace, Vector3};
use collision::Plane;
use crate::scene::camera::Camera;
use crate::scene::projection::Projection;

pub struct Frustum {
    pub near_plane: Plane<f32>,
    pub far_plane: Plane<f32>,
    pub right_plane: Plane<f32>,
    pub left_plane: Plane<f32>,
    pub top_plane: Plane<f32>,
    pub bottom_plane: Plane<f32>,
}

impl Frustum {

    pub fn new(camera: &Camera, projection: &Projection) -> Self {
        let mut frustum = Self {
            near_plane: Plane::from_abcd(0.0, 0.0, 0.0, 0.0),
            far_plane: Plane::from_abcd(0.0, 0.0, 0.0, 0.0),
            right_plane: Plane::from_abcd(0.0, 0.0, 0.0, 0.0),
            left_plane: Plane::from_abcd(0.0, 0.0, 0.0, 0.0),
            top_plane: Plane::from_abcd(0.0, 0.0, 0.0, 0.0),
            bottom_plane: Plane::from_abcd(0.0, 0.0, 0.0, 0.0),
        };

        frustum.update(camera, projection);

        frustum
    }

    pub fn update(&mut self, camera: &Camera, projection: &Projection) {
        let half_v_side = projection.zfar * (projection.fovy * 0.5).tan();
        let half_h_side = half_v_side * projection.aspect;
        let front_times_far = projection.zfar * camera.front();
        let pos = camera.position();
        let right = camera.front().cross(Vector3::unit_y()).normalize();
        let up = camera.up();

        let near_normal = camera.front().normalize();
        let near_face = near_normal.dot(pos + projection.znear * camera.front());
        self.near_plane = Plane::new(near_normal, near_face);

        let far_normal = -camera.front().normalize();
        let far_face = far_normal.dot(pos + front_times_far);
        self.far_plane = Plane::new(far_normal, far_face);

        let right_normal = up.cross(front_times_far + right * half_h_side);
        let right_face = right_normal.dot(pos.clone());
        self.right_plane = Plane::new(right_normal, right_face);

        let left_normal = (front_times_far - right * half_h_side).cross(up.clone());
        let left_face = left_normal.dot(pos.clone());
        self.left_plane = Plane::new(left_normal, left_face);

        let top_normal = right.cross(front_times_far - up * half_v_side);
        let top_face = top_normal.dot(pos.clone());
        self.top_plane = Plane::new(top_normal, top_face);

        let bottom_normal = (front_times_far + up * half_v_side).cross(right);
        let bottom_face = bottom_normal.dot(pos.clone());
        self.bottom_plane = Plane::new(bottom_normal, bottom_face);
    }

    pub fn get(&self) -> collision::Frustum<f32> {
        collision::Frustum::new(
            self.left_plane,
            self.right_plane,
            self.bottom_plane,
            self.top_plane,
            self.near_plane,
            self.far_plane,
        )
    }

}