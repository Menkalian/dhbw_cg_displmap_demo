use std::ffi::{CStr, CString};

use cgmath::{InnerSpace, Matrix4, Vector3};
use gl::types::{GLchar, GLuint};

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
