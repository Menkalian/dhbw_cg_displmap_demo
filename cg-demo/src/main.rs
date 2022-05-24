extern crate cgmath;
extern crate gl;
extern crate image;
extern crate sdl2;

use std::path::Path;

use cgmath::{Matrix4, Vector3};
use gl::types::GLuint;
use log::{debug, info, Level, warn};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::WindowBuildError;
use sdl2::VideoSubsystem;

use crate::glhelper::{Camera, MovementDirection::{BACKWARD, FORWARD, LEFT, RIGHT}, Program, TextureCollection, utils::{calc_projection_matrix, configure_vao, fill_vbo}};
use crate::glhelper::MovementDirection::{DOWN, UP};
use crate::resources::Resources;

pub mod glhelper;
pub mod resources;

const LOG_TARGET: &str = "Main";

const WINDOW_TITLE: &str = "Displacement Map Demo";
const WINDOW_WIDTH: u32 = 900;
const WINDOW_HEIGHT: u32 = 700;

const MIN_X: f32 = -1.0;
const MAX_X: f32 = 1.0;
const MIN_Y: f32 = -1.0;
const MAX_Y: f32 = 1.0;

const SAMPLE_STEPS_X: [f32; 6] = [2.0, 4.0, 16.0, 64.0, 256.0, 1024.0];
const SAMPLE_STEPS_Y: [f32; 6] = [2.0, 4.0, 16.0, 64.0, 256.0, 1024.0];
const SAMPLE_START_IDX: usize = 0;

///
/// Function that is executed when starting the compiled program
///
fn main() {
    simple_logger::init_with_level(Level::Debug).unwrap();

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = configure_and_create_window(&video_subsystem).unwrap();

    // Configure OpenGL to use the SDL2 implementation of the interfaces
    let _gl_context = window.gl_create_context().unwrap();
    let _gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    unsafe {
        // Setup viewport
        gl::Viewport(0, 0,
                     WINDOW_WIDTH as gl::types::GLsizei,
                     WINDOW_HEIGHT as gl::types::GLsizei);
        // Set gray background color
        gl::ClearColor(0.8, 0.8, 0.8, 1.0);

        // Enable features
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
    }

    // Load shader
    let res = Resources::from_relative_exe_path(Path::new("resources")).unwrap();
    let mut state = AppState::new(&res).unwrap();

    // init immutable data
    let demo_texture = TextureCollection::from_resources(&res, "textures/wall", "jpg").unwrap();
    let light_pos: Vector3<f32> = cgmath::vec3(1.0, 1.0, 1.0);
    let model_trans: Matrix4<f32> = cgmath::One::one(); // no transformation for the displayed model; only the camera changes

    log_instructions();

    let mut event_stream = sdl.event_pump().unwrap();
    loop {
        for event in event_stream.poll_iter() {
            handle_event(&mut state, event);
        }

        // Terminate if necessary
        if state.should_terminate {
            break;
        }

        // rendering
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let proj = calc_projection_matrix(state.camera.zoom().to_radians(), (WINDOW_WIDTH as f32) / (WINDOW_HEIGHT as f32), 0.1, 100.0).unwrap();
        let view = state.camera.calc_view_matrix();
        let pos = state.camera.position();

        state.current_program().unwrap().set_active();
        TextureCollection::configure_program(&state.current_program().unwrap());
        demo_texture.set_active();

        // We can only borrow the value here, so the `set_active`-call needs to retrieve it manually
        let current_program = state.current_program().unwrap();

        current_program.set_property_mat4("projection", &proj);
        current_program.set_property_mat4("view", &view);
        current_program.set_property_mat4("model", &model_trans);

        current_program.set_property_vec3("viewPos", &pos);
        current_program.set_property_vec3("lightPos", &light_pos);

        unsafe {
            gl::BindVertexArray(state.vao_id);
            gl::DrawArrays(
                gl::TRIANGLES,
                0,
                state.point_count as gl::types::GLsizei,
            );
        }

        // Swap buffer
        window.gl_swap_window();
    }
}

/// Creates an SDL Window and configures it for use with OpenGl
fn configure_and_create_window(video_sys: &VideoSubsystem) -> Result<sdl2::video::Window, WindowBuildError> {
    // Configure OpenGL attributes
    let gl_attr = video_sys.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    // Initialize Window
    video_sys
        .window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
        .opengl()
        .build()
}

/// Generates vertices for a square with the given sample-size and stores the VBO-data to the buffer.
fn generate_vertices(samples_idx: usize, buffer: &mut Vec<f32>) -> u32 {
    // Clear existing data
    buffer.clear();

    let full_diff_x = MAX_X - MIN_X;
    let full_diff_y = MAX_Y - MIN_Y;
    let diff_x = full_diff_x / SAMPLE_STEPS_X[samples_idx];
    let diff_y = full_diff_y / SAMPLE_STEPS_Y[samples_idx];

    let normal = cgmath::vec3(0.0, 0.0, 1.0);
    let mut point_count: u32 = 0;

    // Generate the data
    for step_x in 0..((SAMPLE_STEPS_X[samples_idx]) as i32) {
        for step_y in 0..((SAMPLE_STEPS_Y[samples_idx]) as i32) {
            // Step 1: Positions
            let p1 = cgmath::vec3(MIN_X + (step_x as f32 * diff_x), MIN_Y + (step_y as f32 * diff_y), 0.0);
            let p2 = cgmath::vec3(MIN_X + (step_x as f32 * diff_x), MIN_Y + (step_y as f32 * diff_y) + diff_y, 0.0);
            let p3 = cgmath::vec3(MIN_X + (step_x as f32 * diff_x) + diff_x, MIN_Y + (step_y as f32 * diff_y) + diff_y, 0.0);
            let p4 = cgmath::vec3(MIN_X + (step_x as f32 * diff_x) + diff_x, MIN_Y + (step_y as f32 * diff_y), 0.0);

            // Step 2: Texture coordinates
            let uv1 = cgmath::vec2((p1.x - MIN_X) / full_diff_x, (p1.y - MIN_Y) / full_diff_y);
            let uv2 = cgmath::vec2((p2.x - MIN_X) / full_diff_x, (p2.y - MIN_Y) / full_diff_y);
            let uv3 = cgmath::vec2((p3.x - MIN_X) / full_diff_x, (p3.y - MIN_Y) / full_diff_y);
            let uv4 = cgmath::vec2((p4.x - MIN_X) / full_diff_x, (p4.y - MIN_Y) / full_diff_y);

            // Step 3: tangent and bitangent
            // Tri 1
            let edge1_t1 = p2 - p1;
            let edge2_t1 = p3 - p1;
            let delta_uv1_t1 = uv2 - uv1;
            let delta_uv2_t1 = uv3 - uv1;

            let f_t1 = 1.0 / (delta_uv1_t1.x * delta_uv2_t1.y - delta_uv2_t1.x * delta_uv1_t1.y);

            let tangent1 = cgmath::vec3(
                f_t1 * (delta_uv2_t1.y * edge1_t1.x - delta_uv1_t1.y * edge2_t1.x),
                f_t1 * (delta_uv2_t1.y * edge1_t1.y - delta_uv1_t1.y * edge2_t1.y),
                f_t1 * (delta_uv2_t1.y * edge1_t1.z - delta_uv1_t1.y * edge2_t1.z),
            );
            let bitangent1 = cgmath::vec3(
                f_t1 * (-1.0 * delta_uv2_t1.y * edge1_t1.x + delta_uv1_t1.y * edge2_t1.x),
                f_t1 * (-1.0 * delta_uv2_t1.y * edge1_t1.y + delta_uv1_t1.y * edge2_t1.y),
                f_t1 * (-1.0 * delta_uv2_t1.y * edge1_t1.z + delta_uv1_t1.y * edge2_t1.z),
            );

            // Tri2
            let edge1_t2 = p3 - p1;
            let edge2_t2 = p4 - p1;
            let delta_uv1_t2 = uv3 - uv1;
            let delta_uv2_t2 = uv4 - uv1;

            let f_t2 = 1.0 / (delta_uv1_t2.x * delta_uv2_t2.y - delta_uv2_t2.x * delta_uv1_t2.y);

            let tangent2 = cgmath::vec3(
                f_t2 * (delta_uv2_t2.y * edge1_t2.x - delta_uv1_t2.y * edge2_t2.x),
                f_t2 * (delta_uv2_t2.y * edge1_t2.y - delta_uv1_t2.y * edge2_t2.y),
                f_t2 * (delta_uv2_t2.y * edge1_t2.z - delta_uv1_t2.y * edge2_t2.z),
            );
            let bitangent2 = cgmath::vec3(
                f_t2 * (-1.0 * delta_uv2_t2.y * edge1_t2.x + delta_uv1_t2.y * edge2_t2.x),
                f_t2 * (-1.0 * delta_uv2_t2.y * edge1_t2.y + delta_uv1_t2.y * edge2_t2.y),
                f_t2 * (-1.0 * delta_uv2_t2.y * edge1_t2.z + delta_uv1_t2.y * edge2_t2.z),
            );

            // Step 4: Add the data
            let mut tmp_buffer = vec![
                // position       // normal                    // tex.-coords // tangent                          // bitangent
                p1.x, p1.y, p1.z, normal.x, normal.y, normal.z, uv1.x, uv1.y, tangent1.x, tangent1.y, tangent1.z, bitangent1.x, bitangent1.y, bitangent1.z,
                p2.x, p2.y, p2.z, normal.x, normal.y, normal.z, uv2.x, uv2.y, tangent1.x, tangent1.y, tangent1.z, bitangent1.x, bitangent1.y, bitangent1.z,
                p3.x, p3.y, p3.z, normal.x, normal.y, normal.z, uv3.x, uv3.y, tangent1.x, tangent1.y, tangent1.z, bitangent1.x, bitangent1.y, bitangent1.z,
                p1.x, p1.y, p1.z, normal.x, normal.y, normal.z, uv1.x, uv1.y, tangent2.x, tangent2.y, tangent2.z, bitangent2.x, bitangent2.y, bitangent2.z,
                p3.x, p3.y, p3.z, normal.x, normal.y, normal.z, uv3.x, uv3.y, tangent2.x, tangent2.y, tangent2.z, bitangent2.x, bitangent2.y, bitangent2.z,
                p4.x, p4.y, p4.z, normal.x, normal.y, normal.z, uv4.x, uv4.y, tangent2.x, tangent2.y, tangent2.z, bitangent2.x, bitangent2.y, bitangent2.z,
            ];
            buffer.append(&mut tmp_buffer);
            point_count += 6;
        }
    }

    point_count
}

fn handle_event(state: &mut AppState, event: Event) {
    // Input handling
    match event {
        Event::Quit { .. } => state.terminate(),
        Event::KeyDown { keycode, .. } => {
            match keycode.unwrap_or_else(|| Keycode::F24) { // Match unknown keys to the (unused) F24-Key
                Keycode::Plus | Keycode::KpPlus => {
                    state.increase_samples()
                }
                Keycode::Minus | Keycode::KpMinus => {
                    state.decrease_samples()
                }
                Keycode::W | Keycode::Up => {
                    state.camera.move_camera(FORWARD, 0.1)
                }
                Keycode::A | Keycode::Left => {
                    state.camera.move_camera(LEFT, 0.1)
                }
                Keycode::S | Keycode::Down => {
                    state.camera.move_camera(BACKWARD, 0.1)
                }
                Keycode::D | Keycode::Right => {
                    state.camera.move_camera(RIGHT, 0.1)
                }
                Keycode::Space | Keycode::PageUp => {
                    state.camera.move_camera(UP, 0.1)
                }
                Keycode::LCtrl | Keycode::PageDown => {
                    state.camera.move_camera(DOWN, 0.1)
                }
                Keycode::Kp0 | Keycode::Home => {
                    state.camera.reset_position()
                }
                Keycode::M => {
                    state.cycle_programs();
                }
                Keycode::Escape => state.terminate(),
                _ => {}
            }
        }
        Event::MouseMotion { xrel, yrel, .. } => state.camera.rotate_camera(xrel as f32, yrel as f32),
        Event::MouseWheel { y, .. } => state.camera.zoom_camera(y as f32),
        _ => {} // do nothing for unhandled events
    }
}

fn log_instructions() {
    // Instructions for using
    warn!(target: "INSTRUCTIONS", r#"
    Controls:
     - ESC           => Quit
     - '+'           => Increase model vertices
     - '-'           => Decrease model vertices
     - W/UP          => Move forward
     - A/LEFT        => Move left
     - S/DOWN        => Move backward
     - S/RIGHT       => Move right
     - Space/PgUp    => Move up
     - Ctrl/PgDown   => Move down
     - Pos1/KeyPad0  => Reset camera
     - M             => Cycle shaders

    Use the mouse to look around.
    Scroll to zoom.
    "#);
}

/// # AppState
/// Contains values related to the mutable state of the application
struct AppState {
    /// Camera to view the scene
    camera: Camera,

    /// Flag to terminate the program
    should_terminate: bool,

    /// Index of the active program/shaders
    used_program_idx: usize,
    /// List of available programs/shaders
    available_programs: Vec<Program>,
    /// List of readable identifiers for the available programs/shaders
    available_program_names: Vec<String>,

    /// Index to determine the amount of samples to generate
    samples_idx: usize,

    /// OpenGL-Id of the VBO
    vbo_id: GLuint,
    /// OpenGL-Id of the VAO
    vao_id: GLuint,
    /// Current count of vertices
    point_count: u32,
}

impl AppState {
    /// Initialize the AppState with default values
    fn new(res: &Resources) -> Result<AppState, String> {
        let mut state = AppState {
            camera: Camera::new(),
            should_terminate: false,

            used_program_idx: 0,
            available_programs: Vec::new(),
            available_program_names: Vec::new(),

            samples_idx: SAMPLE_START_IDX,

            vbo_id: 0,
            vao_id: 0,
            point_count: 0,
        };

        // Load and initialize programs
        state.available_program_names.push("Kein Mapping".to_string());
        state.available_programs.push(
            Program::from_res(res, "shaders/base")
                .map_err(|s| s)?);

        state.available_program_names.push("Normal-Mapping".to_string());
        state.available_programs.push(
            Program::from_res(res, "shaders/normal")
                .map_err(|s| s)?);

        // Init buffers
        unsafe {
            gl::GenBuffers(1, &mut state.vbo_id);
        }
        state.refresh_vbo();
        state.vao_id = configure_vao(state.vbo_id);

        Ok(state)
    }

    pub fn terminate(&mut self) {
        self.should_terminate = true;
    }

    pub fn cycle_programs(&mut self) {
        self.used_program_idx = (self.used_program_idx + 1) % self.available_programs.len();
        info!(target: LOG_TARGET, "Using program {}: \"{}\"", self.used_program_idx, self.available_program_names.get(self.used_program_idx).unwrap());
    }

    pub fn current_program(&mut self) -> Option<&mut Program> {
        self.available_programs.get_mut(self.used_program_idx)
    }

    pub fn increase_samples(&mut self) {
        self.samples_idx = (self.samples_idx + 1).clamp(0, SAMPLE_STEPS_X.len() - 1);
        info!(target: LOG_TARGET, "Using sample amount {}: {}x{}", self.samples_idx, SAMPLE_STEPS_X[self.samples_idx], SAMPLE_STEPS_Y[self.samples_idx]);
        self.refresh_vbo();
    }

    pub fn decrease_samples(&mut self) {
        if self.samples_idx == 0 {
            debug!("Sample amount could not be decreased");
            return;
        }

        self.samples_idx = (self.samples_idx - 1).clamp(0, SAMPLE_STEPS_X.len() - 1);
        info!(target: LOG_TARGET, "Using sample amount {}: {}x{}", self.samples_idx, SAMPLE_STEPS_X[self.samples_idx], SAMPLE_STEPS_Y[self.samples_idx]);
        self.refresh_vbo();
    }

    fn refresh_vbo(&mut self) {
        let mut vertices = Vec::new();
        self.point_count = generate_vertices(self.samples_idx, &mut vertices);
        fill_vbo(self.vbo_id, &vertices);
    }
}
