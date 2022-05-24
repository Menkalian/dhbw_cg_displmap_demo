use cgmath::InnerSpace;
use log::{debug, info, trace};

use crate::glhelper::utils::calc_look_at_matrix;

const LOG_TARGET: &str = "Camera";

/// # Camera
/// Contains all necessary values to calculate the view position, direction and zoom.
#[derive(Debug)]
pub struct Camera {
    /// Current position of the camera in cartesian coordinates
    pos: cgmath::Vector3<f32>,
    /// Direction of the **front** facing side of the camera (directional vector)
    front: cgmath::Vector3<f32>,
    /// Direction of the **up** facing side of the camera (directional vector)
    up: cgmath::Vector3<f32>,
    /// Direction of the **right** facing side of the camera (directional vector)
    right: cgmath::Vector3<f32>,
    /// Relative up direction of the world. used to calculate the directional vectors
    world_up: cgmath::Vector3<f32>,

    /// yaw of the camera in degrees  (horizontal (xz) rotation)
    yaw: f32,
    /// pitch of the camera in degrees (vertical (yz) rotation)
    pitch: f32,

    /// Current zoom of the camera in degrees (= fovy-angle of the camera, sensible values are 30-60 degrees)
    zoom: f32,

    /// Movement speed of the camera
    movement_speed: f32,
    /// Sensitivity to mouse movement
    mouse_sens: f32,
}

/// Possible movement directions to control the camera
#[derive(Debug)]
pub enum MovementDirection {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

impl Camera {
    /// # Constructor
    /// Creates a new Camera with default values.
    /// The assumed `world_up` is `(0,1,0)`
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
        info!(target: LOG_TARGET, "Created new camera: {:?}", to_return);
        to_return
    }

    /// Calculates the view matrix for the current values of the camera
    pub fn calc_view_matrix(&self) -> cgmath::Matrix4<f32> {
        calc_look_at_matrix(self.pos, self.pos + self.front, self.up)
    }

    /// Get the current value for the zoom
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Get the current position in cartesian coordinates
    pub fn position(&self) -> cgmath::Vector3<f32> {
        self.pos
    }

    /// Resets the camera to the default position, direction and zoom
    pub fn reset_position(&mut self) {
        self.pos = cgmath::vec3(0.0, 0.0, -1.0);
        self.yaw = 90.0;
        self.pitch = 0.0;
        self.zoom = 45.0;
        self.recalculate_direction_vectors();
    }

    /// Moves the camera in the given direction
    pub fn move_camera(&mut self, dir: MovementDirection, amount: f32) {
        let v = self.movement_speed * amount;
        debug!(target: LOG_TARGET, "Moving {:?} by {} units", dir, v);

        match dir {
            MovementDirection::FORWARD => {
                self.pos += self.front * v
            }
            MovementDirection::BACKWARD => {
                self.pos -= self.front * v
            }
            MovementDirection::LEFT => {
                self.pos -= self.right * v
            }
            MovementDirection::RIGHT => {
                self.pos += self.right * v
            }
            MovementDirection::UP => {
                self.pos += self.up * v
            }
            MovementDirection::DOWN => {
                self.pos -= self.up * v
            }
        }

        trace!(target: LOG_TARGET, "New position: {:?}", self.pos);
    }

    /// Rotates the camera by the given amount
    pub fn rotate_camera(&mut self, horiz_amount: f32, vert_amount: f32) {
        debug!(target: LOG_TARGET, "Rotating by {}° horizontally and {}° vertically.",
            horiz_amount * self.mouse_sens, vert_amount* self.mouse_sens);

        self.yaw = self.yaw + horiz_amount * self.mouse_sens;
        self.pitch = (self.pitch + vert_amount * self.mouse_sens).clamp(-89.9, 89.9);
        self.recalculate_direction_vectors();
        trace!(target: LOG_TARGET, "New rotation: yaw: {}°, pitch: {}°. [front: {:?}; up: {:?}, right: {:?}]", self.yaw, self.pitch, self.front, self.up, self.right);
    }

    /// Zooms the camera by the given amount
    pub fn zoom_camera(&mut self, delta: f32) {
        debug!(target: LOG_TARGET, "Zooming by {}°.", delta);
        self.zoom = (self.zoom - delta).clamp(10.0, 60.0);
        trace!(target: LOG_TARGET, "New zoom: {}°", self.zoom);
    }

    /// Used to recalculate the directional vectors from `yaw` and `pitch`
    fn recalculate_direction_vectors(&mut self) {
        self.front = cgmath::vec3(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        ).normalize();

        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }
}
