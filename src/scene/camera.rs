use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use cgmath::{EuclideanSpace, InnerSpace, Matrix4, Point3, Rad, vec3, Vector3, Zero};
use winit::event::{ElementState, MouseButton, VirtualKeyCode};

pub struct Camera {
    position: Vector3<f32>,
    front: Vector3<f32>,
    up: Vector3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,

    yaw_delta: Rad<f32>,
    pitch_delta: Rad<f32>,

    sensitivity: f32,

    movement_speed: f32,

    moving_forward: bool,
    moving_backward: bool,
    moving_left: bool,
    moving_right: bool,
    moving_up: bool,
    moving_down: bool,

    is_sprinting: bool,
}

impl Camera {

    #[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    );

    pub const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

    pub fn new<
        V: Into<Vector3<f32>>,
        Y: Into<Rad<f32>>,
        P: Into<Rad<f32>>,
    >(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            front: Vector3::zero(),
            up: Vector3::zero(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            yaw_delta: Rad::zero(),
            pitch_delta: Rad::zero(),
            sensitivity: 1.0,
            movement_speed: 3.0,
            moving_forward: false,
            moving_backward: false,
            moving_left: false,
            moving_right: false,
            moving_up: false,
            moving_down: false,
            is_sprinting: false,
        }
    }

    pub fn yaw(&self) -> &Rad<f32> {
        &self.yaw
    }

    pub fn set_yaw(&mut self, yaw: Rad<f32>) {
        self.yaw = yaw;
    }

    pub fn add_yaw(&mut self, yaw: Rad<f32>) {
        self.yaw += yaw;
    }

    pub fn pitch(&self) -> &Rad<f32> {
        &self.pitch
    }

    pub fn set_pitch(&mut self, pitch: Rad<f32>) {
        self.pitch = pitch;
    }

    pub fn add_pitch(&mut self, pitch: Rad<f32>) {
        self.pitch += pitch;
    }

    pub fn position(&self) -> &Vector3<f32> {
        &self.position
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }

    pub fn front(&self) -> &Vector3<f32> {
        &self.front
    }

    pub fn set_front(&mut self, front: Vector3<f32>) {
        self.front = front;
    }

    pub fn up(&self) -> &Vector3<f32> {
        &self.up
    }

    pub fn set_up(&mut self, up: Vector3<f32>) {
        self.up = up;
    }

    pub fn translate(&mut self, translation: Vector3<f32>) {
        self.position += translation;
    }

    pub fn translate_x(&mut self, x: f32) {
        self.position.x += x;
    }

    pub fn translate_y(&mut self, y: f32) {
        self.position.y += y;
    }

    pub fn translate_z(&mut self, z: f32) {
        self.position.z += z;
    }

    pub fn update(&mut self, dt: Duration) {
        let dt = dt.as_secs_f32();

        self.add_yaw(self.yaw_delta * self.sensitivity * dt);
        self.add_pitch(-self.pitch_delta * self.sensitivity * dt);

        self.yaw_delta = Rad::zero();
        self.pitch_delta = Rad::zero();

        if self.pitch < -Rad(Self::SAFE_FRAC_PI_2) {
            self.pitch = -Rad(Self::SAFE_FRAC_PI_2);
        } else if self.pitch > Rad(Self::SAFE_FRAC_PI_2) {
            self.pitch = Rad(Self::SAFE_FRAC_PI_2);
        }

        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let front = vec3(
            sin_yaw * cos_pitch,
            sin_pitch,
            -cos_yaw * cos_pitch,
        ).normalize();

        let right = front.cross(Vector3::unit_y()).normalize();

        let amount_forward = if self.moving_forward {
            self.movement_speed * if self.is_sprinting { 4.0 } else { 1.0 }
        } else {
            0.0
        };

        let amount_back = if self.moving_backward { self.movement_speed } else { 0.0 };
        let amount_left = if self.moving_left { self.movement_speed } else { 0.0 };
        let amount_right = if self.moving_right { self.movement_speed } else { 0.0 };
        let amount_up = if self.moving_up { self.movement_speed } else { 0.0 };
        let amount_down = if self.moving_down { self.movement_speed } else { 0.0 };

        self.translate(front * (amount_forward - amount_back) * dt);
        self.translate(right * (amount_right - amount_left) * dt);

        self.translate_y((amount_up - amount_down) * dt);

        self.set_front(front);
        self.set_up(right.cross(front).normalize());
    }

    pub fn process_mouse_motion(&mut self, dx: f64, dy: f64) {
        self.yaw_delta = Rad(dx as f32);
        self.pitch_delta = Rad(dy as f32);
    }

    pub fn process_key_input(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let pressed = state == ElementState::Pressed;

        match key {
            VirtualKeyCode::W => self.moving_forward = pressed,
            VirtualKeyCode::A => self.moving_left = pressed,
            VirtualKeyCode::S => self.moving_backward = pressed,
            VirtualKeyCode::D => self.moving_right = pressed,
            VirtualKeyCode::Space => self.moving_up = pressed,
            VirtualKeyCode::LControl => self.moving_down = pressed,
            VirtualKeyCode::LShift => self.is_sprinting = pressed,
            _ => return false,
        }

        true
    }

    pub fn process_mouse_input(&mut self, button: MouseButton, state: ElementState) -> bool {
        false
    }

    pub fn calculate_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_at(
            Point3::from_vec(self.position),
            Point3::from_vec(Vector3::new(
                sin_yaw * cos_pitch,
                sin_pitch,
                -cos_yaw * cos_pitch
            ).normalize() + self.position),
            Vector3::unit_y(),
        )
    }

}