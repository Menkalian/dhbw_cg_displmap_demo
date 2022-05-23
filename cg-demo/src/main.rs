extern crate cgmath;
extern crate gl;
extern crate sdl2;

use std::path::Path;

use cgmath::{Matrix4, Vector3};
use sdl2::keyboard::Keycode;
use sdl2::video::WindowBuildError;
use sdl2::VideoSubsystem;

use resources::Resources;

use crate::camera::Camera;
use crate::camera::Direction::{BACKWARD, FORWARD, LEFT, RIGHT};
use crate::render_gl::TextureCollection;

pub mod render_gl;
pub mod camera;
pub mod resources;

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
    simple_logger::init().unwrap();

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
    let shader_program = render_gl::Program::from_res(&res, "shaders/base").unwrap();
    shader_program.set_active();

    // Load texture
    TextureCollection::configure_program(&shader_program);
    let demo_texture = TextureCollection::from_resources(&res, "textures/wall", "jpg").unwrap();
    demo_texture.set_active();

    let mut camera = Camera::new();

    // Generate initial data
    let mut sample_idx = SAMPLE_START_IDX;
    let mut vertices: Vec<f32> = Vec::new();
    let mut point_count: u32;
    let mut vbo: gl::types::GLuint = 0;

    // init static data
    let light_pos: Vector3<f32> = cgmath::vec3(0.0, 0.0, 1.0);
    let model_trans: Matrix4<f32> = cgmath::One::one(); // no transformation for the displayed model; only the camera changes

    unsafe {
        gl::GenBuffers(1, &mut vbo);
    }

    point_count = generate_verticies(sample_idx, &mut vertices);
    fill_vbo(vbo, &vertices);
    let vao = configure_vao(vbo);

    let mut event_stream = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_stream.poll_iter() {
            // Input handling
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::KeyDown { keycode, .. } => {
                    match keycode.unwrap_or_else(|| Keycode::F24) { // Match unknown keys to the (unused) F24-Key
                        Keycode::Plus | Keycode::KpPlus => {
                            sample_idx = (sample_idx + 1).clamp(0, SAMPLE_STEPS_X.len() - 1);
                            point_count = generate_verticies(sample_idx, &mut vertices);
                            fill_vbo(vbo, &mut vertices)
                        }
                        Keycode::Minus | Keycode::KpMinus => {
                            sample_idx = (sample_idx.max(1) - 1).clamp(0, SAMPLE_STEPS_X.len() - 1);
                            point_count = generate_verticies(sample_idx, &mut vertices);
                            fill_vbo(vbo, &vertices)
                        }
                        Keycode::W | Keycode::Up => {
                            camera.process_keyboard(FORWARD, 0.1)
                        }
                        Keycode::A | Keycode::Left => {
                            camera.process_keyboard(LEFT, 0.1)
                        }
                        Keycode::S | Keycode::Down => {
                            camera.process_keyboard(BACKWARD, 0.1)
                        }
                        Keycode::D | Keycode::Right => {
                            camera.process_keyboard(RIGHT, 0.1)
                        }
                        Keycode::O | Keycode::Home => {
                            camera.reset_position()
                        }
                        Keycode::Escape => break 'main,
                        _ => {}
                    }
                }
                sdl2::event::Event::MouseMotion { xrel, yrel, .. } => camera.process_mouse_move(xrel as f32, yrel as f32),
                sdl2::event::Event::MouseWheel { y, .. } => camera.process_mouse_scroll(y as f32 / 10.0),
                _ => {} // do nothing for unhandled events
            }
        }

        // rendering
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let proj = build_projection_matrix(camera.zoom(), (WINDOW_WIDTH as f32) / (WINDOW_HEIGHT as f32), 0.1, 100.0).unwrap();

        shader_program.set_active();
        TextureCollection::configure_program(&shader_program);
        demo_texture.set_active();

        shader_program.set_property_mat4("projection", &proj);
        shader_program.set_property_mat4("view", &camera.calc_view_matrix());
        shader_program.set_property_mat4("model", &model_trans);

        shader_program.set_property_vec3("viewPos", &camera.position());
        shader_program.set_property_vec3("lightPos", &light_pos);

        unsafe {
            gl::BindVertexArray(vao);
            gl::DrawArrays(
                gl::TRIANGLES,
                0,
                point_count as gl::types::GLsizei,
            );
        }

        // Swap buffer
        window.gl_swap_window();
    }
}

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

fn fill_vbo(vbo_id: gl::types::GLuint, data: &Vec<f32>) {
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo_id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            data.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }
}

fn generate_verticies(samples_idx: usize, buffer: &mut Vec<f32>) -> u32 {
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

fn configure_vao(vbo_id: gl::types::GLuint) -> gl::types::GLuint {
    let mut vao: gl::types::GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo_id);

        // Configure the following layout:
        //   layout (location = 0) in vec3  inPos;
        //   layout (location = 1) in vec3  inNormal;
        //   layout (location = 2) in vec2  inTexCoords;
        //   layout (location = 3) in vec3  inTangent;
        //   layout (location = 4) in vec3  inBitangent;
        //
        // sum of components = 14
        // since float / f32 is used, all the values are tightly packed
        let stride = (14 * std::mem::size_of::<f32>()) as gl::types::GLint;

        // Position
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            3, gl::FLOAT, gl::FALSE, // amount and type of data
            stride, std::ptr::null(),
        );

        // Normal
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            3, gl::FLOAT, gl::FALSE,
            stride, calc_f32_offset(3),
        );

        // TexCoords
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(
            2,
            2, gl::FLOAT, gl::FALSE,
            stride, calc_f32_offset(6),
        );

        // Tangent
        gl::EnableVertexAttribArray(3);
        gl::VertexAttribPointer(
            3,
            3, gl::FLOAT, gl::FALSE,
            stride, calc_f32_offset(8),
        );

        // Bi-Tangent
        gl::EnableVertexAttribArray(4);
        gl::VertexAttribPointer(
            4,
            3, gl::FLOAT, gl::FALSE,
            stride, calc_f32_offset(11),
        );

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    vao
}

fn calc_f32_offset(amount: usize) -> *const gl::types::GLvoid {
    (amount * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
}

fn build_projection_matrix(fovy: f32, aspect: f32, z_near: f32, z_far: f32) -> Result<Matrix4<f32>, String> {
    if aspect == 0.0 {
        return Err("Aspect ratio may not be 0".to_string());
    }
    if z_far == z_near {
        return Err("z-values may not be the same".to_string());
    }

    let tan_half_fovy = (fovy / 2.0).tan();
    let mut result: Matrix4<f32> = cgmath::Zero::zero();
    result.x.x = 1.0 / (aspect * tan_half_fovy);
    result.y.y = 1.0 / tan_half_fovy;
    result.z.z = -1.0 * (z_far + z_near) / (z_far - z_near);
    result.z.w = -1.0;
    result.w.z = (-2.0 * z_far * z_near) / (z_far - z_near);

    Ok(result)
}
