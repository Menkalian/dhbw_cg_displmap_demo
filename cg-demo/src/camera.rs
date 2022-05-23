use cgmath;
use cgmath::{InnerSpace, Matrix4};

pub struct Camera {
    pos: cgmath::Vector3<f32>,
    front: cgmath::Vector3<f32>,
    up: cgmath::Vector3<f32>,
    right: cgmath::Vector3<f32>,
    world_up: cgmath::Vector3<f32>,

    yaw: f32,
    pitch: f32,

    movement_speed: f32,
    mouse_sens: f32,
    zoom: f32,
}

pub enum Direction {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT
}

impl Camera {
    pub fn new() -> Camera {
        let mut to_return = Camera {
            pos: cgmath::Zero::zero(),
            front: cgmath::Zero::zero(),
            up: cgmath::Zero::zero(),
            right: cgmath::Zero::zero(),
            world_up: cgmath::vec3(0.0, 1.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            movement_speed: 0.5,
            mouse_sens: 0.1,
            zoom: 0.0,
        };
        to_return.reset_position();
        to_return
    }

    pub fn calc_view_matrix(&self) -> Matrix4<f32> {
        calc_look_at_matrix(self.pos, self.pos + self.front, self.up)
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn position(&self) -> cgmath::Vector3<f32> {
        self.pos
    }

    pub fn reset_position(&mut self) {
        self.pos = cgmath::vec3(0.0, 0.0, -1.0);
        self.yaw = 90.0;
        self.pitch = 0.0;
        self.zoom = 1.5;
        self.recalculate_vectors();
    }

    pub fn process_keyboard(&mut self, dir: Direction, delta_t: f32) {
        let v = self.movement_speed * delta_t;

        match dir {
            Direction::FORWARD => {
                self.pos += self.front * v
            }
            Direction::BACKWARD => {
                self.pos -= self.front * v
            }
            Direction::LEFT => {
                self.pos -= self.right * v
            }
            Direction::RIGHT => {
                self.pos += self.right * v
            }
        }
    }

    pub fn process_mouse_move(&mut self, dx: f32, dy: f32) {
        self.yaw = self.yaw + dx * self.mouse_sens;
        self.pitch = (self.pitch + dy * self.mouse_sens).clamp(-89.1, 89.1);
        self.recalculate_vectors();
    }

    pub fn process_mouse_scroll(&mut self, dy: f32) {
        self.zoom = (self.zoom - dy).clamp(0.1, 2.0);
    }

    fn recalculate_vectors(&mut self) {
        self.front = cgmath::vec3(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        ).normalize();

        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }
}

fn calc_look_at_matrix(eye_pos: cgmath::Vector3<f32>, target: cgmath::Vector3<f32>, up: cgmath::Vector3<f32>) -> Matrix4<f32> {
    let f = (target - eye_pos).normalize();
    let mut u = up.normalize();
    let s = f.cross(u).normalize();
    u = s.cross(f);

    let mut result: Matrix4<f32> = cgmath::One::one();
    result.x.x = s.x;
    result.y.x = s.y;
    result.z.x = s.z;
    result.x.y = u.x;
    result.y.y = u.y;
    result.z.y = u.z;
    result.x.z = -1.0 * f.x;
    result.y.z = -1.0 * f.y;
    result.z.z = -1.0 * f.z;
    result.w.x = -1.0 * s.dot(eye_pos);
    result.w.y = -1.0 * u.dot(eye_pos);
    result.w.z = f.dot(eye_pos);

    result
}
