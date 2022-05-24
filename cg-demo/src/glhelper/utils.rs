use std::ffi::{CStr, CString};

use cgmath::{InnerSpace, Matrix4, Vector3};
use gl::types::{GLchar, GLuint, GLvoid};

/// Compiles shader source code and loads them in OpenGL
pub fn compile_shader_from_source(source: &CStr, kind: GLuint) -> Result<GLuint, String> {
    let id = unsafe {
        gl::CreateShader(kind)
    };

    unsafe {
        gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error: CString = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl::GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut GLchar,
            );
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}

/// Loads an image from the given path and creates an OpenGL texture for it
pub fn load_texture_from_path(img_path: &std::path::Path) -> Result<GLuint, String> {
    let img = image::open(img_path)
        .map_err(|_| "Could not load texture")?;

    let gl_texture_format: gl::types::GLenum;
    match img.color() {
        image::ColorType::L8 => gl_texture_format = gl::RED,
        image::ColorType::Rgb8 => gl_texture_format = gl::RGB,
        image::ColorType::Rgba8 => gl_texture_format = gl::RGBA,
        _ => return Err("Unknown color type".to_string())
    }

    let mut texture_id: GLuint = 0;
    unsafe {
        gl::GenTextures(1, &mut texture_id);
        gl::ActiveTexture(gl::TEXTURE9);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl_texture_format as i32,
                       img.width() as i32, img.height() as i32,
                       0, gl_texture_format, gl::UNSIGNED_BYTE,
                       img.as_bytes().as_ptr().cast());

        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as gl::types::GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as gl::types::GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as gl::types::GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as gl::types::GLint);
    }
    Ok(texture_id)
}

/// Calculates a view matrix that looks from the given position at the target
pub fn calc_look_at_matrix(eye_pos: Vector3<f32>, target: Vector3<f32>, up: Vector3<f32>) -> Matrix4<f32> {
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

/// Calculates a projection matrix with the given fovy and near/far planes
pub fn calc_projection_matrix(fovy: f32, aspect: f32, z_near: f32, z_far: f32) -> Result<Matrix4<f32>, String> {
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

/// Formats texture paths as `{name}_{type}.{ext}` to allow loading belonging textures at once
pub fn format_texture_path(texture_name: &str, texture_type: &str, extension: &str) -> String {
    format!("{}_{}.{}", texture_name, texture_type, extension)
}

/// Create a blank string, that can be used to copy data from C-libraries
pub fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate string buffer
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}

/// Copies the given data to the VBO
pub fn fill_vbo(vbo_id: GLuint, data: &Vec<f32>) {
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo_id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            data.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }
}

/// Configures the VAO for the used layout and assigns it to the VBO
pub fn configure_vao(vbo_id: GLuint) -> GLuint {
    let mut vao: GLuint = 0;
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

/// Calculates the byte-offset for the given amount of `f32`-values
fn calc_f32_offset(amount: usize) -> *const GLvoid {
    (amount * std::mem::size_of::<f32>()) as *const GLvoid
}
