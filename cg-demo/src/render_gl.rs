extern crate image;

use std;
use std::ffi::{CStr, CString};
use std::path::Path;

use cgmath::{Array, Matrix};
use gl;
use image::ColorType;

use crate::Resources;

pub struct Program {
    id: gl::types::GLuint,
}

impl Program {
    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl::CreateProgram() };

        for shader in shaders {
            unsafe { gl::AttachShader(program_id, shader.id()); }
        }

        unsafe { gl::LinkProgram(program_id); }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe { gl::DetachShader(program_id, shader.id()); }
        }

        Ok(Program { id: program_id })
    }

    pub fn from_res(res: &Resources, name: &str) -> Result<Program, String> {
        const POSSIBLE_EXT: [&str; 2] = [
            ".vert",
            ".frag",
        ];

        let shaders = POSSIBLE_EXT.iter()
            .map(|file_extension| {
                Shader::from_res(res, &format!("{}{}", name, file_extension))
            })
            .collect::<Result<Vec<Shader>, String>>()?;

        Program::from_shaders(&shaders[..])
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn set_active(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn set_property_int(&self, property_name: &str, value: i32) {
        unsafe {
            gl::Uniform1i(self.get_uniform_location(property_name).unwrap(), value);
        }
    }

    pub fn set_property_uint(&self, property_name: &str, value: u32) {
        unsafe {
            gl::Uniform1ui(self.get_uniform_location(property_name).unwrap(), value);
        }
    }

    pub fn set_property_mat4(&self, property_name: &str, value: &cgmath::Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(self.get_uniform_location(property_name).unwrap(), 1, gl::FALSE, value.as_ptr());
        }
    }

    pub fn set_property_vec3(&self, property_name: &str, value: &cgmath::Vector3<f32>) {
        unsafe {
            gl::Uniform3fv(self.get_uniform_location(property_name).unwrap(), 1, value.as_ptr());
        }
    }

    fn get_uniform_location(&self, property_name: &str) -> Result<gl::types::GLint, String> {
        let name = CString::new(property_name)
            .map_err(|_| "Could not load CString")?;
        unsafe {
            Ok(gl::GetUniformLocation(self.id, name.as_bytes_with_nul().as_ptr().cast()))
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    pub fn from_source(
        souce: &CStr,
        kind: gl::types::GLenum,
    ) -> Result<Shader, String> {
        let id = shader_from_source(souce, kind)?;
        Ok(Shader { id })
    }

    pub fn from_res(res: &Resources, name: &str) -> Result<Shader, String> {
        // Possible Shader extensions to look for
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] = [
            (".vert", gl::VERTEX_SHADER),
            (".frag", gl::FRAGMENT_SHADER),
        ];

        let shader_kind = POSSIBLE_EXT.iter()
            .find(|&&(file_extension, _)| {
                name.ends_with(file_extension)
            })
            .map(|&(_, kind)| kind)
            .ok_or_else(|| format!("Can not determine shader type for resource {}", name))?;

        let source = res.load_cstring(name)
            .map_err(|e| format!("Error loading resource {}: {:?}", name, e))?;

        Shader::from_source(&source, shader_kind)
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

pub struct TextureCollection {
    base: Texture,
    normal: Texture,
    height: Texture,
}

impl TextureCollection {
    pub fn configure_program(program: &Program) {
        program.set_property_int("baseMap", 0);
        program.set_property_int("normalMap", 1);
        program.set_property_int("heightMap", 2);
    }

    pub fn from_resources(res: &Resources, texture_name: &str, extension: &str) -> Result<TextureCollection, String> {
        let base = Texture::from_resources(&res, format_texture_path(texture_name, "base", extension).as_str()).unwrap();
        let normal = Texture::from_resources(&res, format_texture_path(texture_name, "normal", extension).as_str()).unwrap();
        let height = Texture::from_resources(&res, format_texture_path(texture_name, "height", extension).as_str()).unwrap();

        Ok(TextureCollection {
            base,
            normal,
            height,
        })
    }

    pub fn set_active(&self) {
        self.base.bind_texture(gl::TEXTURE0);
        self.normal.bind_texture(gl::TEXTURE1);
        self.height.bind_texture(gl::TEXTURE2);
    }
}

pub struct Texture {
    id: gl::types::GLuint,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

impl Texture {
    pub fn from_resources(res: &Resources, res_name: &str) -> Result<Texture, String> {
        texture_from_path(
            res.construct_path(res_name)
                .map_err(|_| "Could not create path to resource")?.as_path()
        ).map(|id| Texture { id })
    }

    pub fn bind_texture(&self, texture_unit: gl::types::GLenum) {
        unsafe {
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

fn shader_from_source(source: &CStr, kind: gl::types::GLuint) -> Result<gl::types::GLuint, String> {
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
                error.as_ptr() as *mut gl::types::GLchar,
            );
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}

fn texture_from_path(img_path: &Path) -> Result<gl::types::GLuint, String> {
    let img = image::open(img_path).unwrap();
    // .map_err(|_| "Could not load texture")?;

    let gl_texture_format: gl::types::GLenum;
    match img.color() {
        ColorType::L8 => gl_texture_format = gl::RED,
        ColorType::Rgb8 => gl_texture_format = gl::RGB,
        ColorType::Rgba8 => gl_texture_format = gl::RGBA,
        _ => return Err("Unknown color type".to_string())
    }

    let mut texture_id: gl::types::GLuint = 0;
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

fn format_texture_path(texture_name: &str, texture_type: &str, extension: &str) -> String {
    format!("{}_{}.{}", texture_name, texture_type, extension)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate string buffer
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}
