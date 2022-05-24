use cgmath::{Array, Matrix, Matrix4, Vector3};
use gl::types::{GLint, GLuint};
use log::{debug, info, trace};

use crate::glhelper::Shader;
use crate::glhelper::utils::create_whitespace_cstring_with_len;
use crate::Resources;

const LOG_TARGET: &str = "GlProgram";

/// # Program
/// Handle for an OpenGL-Program.
/// Used to select shaders and transfer data to the shaders
pub struct Program {
    id: GLuint,
}

impl Program {
    /// # Constructor
    /// Creates a new program associated with the given shaders
    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl::CreateProgram() };
        info!(target: LOG_TARGET, "Created new Program: {}", program_id);

        for shader in shaders {
            unsafe { gl::AttachShader(program_id, shader.id()); }
            debug!(target: LOG_TARGET, "Assigning Shader {} to program {}", shader.id(), program_id);
        }

        debug!(target: LOG_TARGET, "Linking shaders in program {}", program_id);
        unsafe { gl::LinkProgram(program_id); }

        let mut success: GLint = 1;
        unsafe {
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: GLint = 0;
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

    /// # Constructor
    /// Creates a new program associated with all shaders that have the given name
    pub fn from_res(res: &Resources, name: &str) -> Result<Program, String> {
        const POSSIBLE_EXT: [&str; 2] = [
            ".vert",
            ".frag",
        ];

        info!(target: LOG_TARGET, "Creating program with shaders \"{}\"", name);
        let shaders = POSSIBLE_EXT.iter()
            .map(|file_extension| {
                Shader::from_res(res, &format!("{}{}", name, file_extension))
            })
            .collect::<Result<Vec<Shader>, String>>()?;

        Program::from_shaders(&shaders[..])
    }

    /// Get id of the program
    pub fn id(&self) -> GLuint {
        self.id
    }

    /// Activate the program
    pub fn set_active(&self) {
        trace!(target: LOG_TARGET, "Setting program {} active.", self.id);
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    /// Setting an `int`-Property for the shaders
    pub fn set_property_int(&self, property_name: &str, value: i32) {
        trace!(target: LOG_TARGET, "Setting property \"{}\" for program {} to {:?}.", property_name, self.id, value);
        unsafe {
            gl::Uniform1i(self.get_uniform_location(property_name).unwrap(), value);
        }
    }

    /// Setting an `uint`-Property for the shaders
    pub fn set_property_uint(&self, property_name: &str, value: u32) {
        trace!(target: LOG_TARGET, "Setting property \"{}\" for program {} to {:?}.", property_name, self.id, value);
        unsafe {
            gl::Uniform1ui(self.get_uniform_location(property_name).unwrap(), value);
        }
    }

    /// Setting an `mat4`-Property for the shaders
    pub fn set_property_mat4(&self, property_name: &str, value: &Matrix4<f32>) {
        trace!(target: LOG_TARGET, "Setting property \"{}\" for program {} to {:?}.", property_name, self.id, value);
        unsafe {
            gl::UniformMatrix4fv(self.get_uniform_location(property_name).unwrap(), 1, gl::FALSE, value.as_ptr());
        }
    }

    /// Setting an `vec3`-Property for the shaders
    pub fn set_property_vec3(&self, property_name: &str, value: &Vector3<f32>) {
        trace!(target: LOG_TARGET, "Setting property \"{}\" for program {} to {:?}.", property_name, self.id, value);
        unsafe {
            gl::Uniform3fv(self.get_uniform_location(property_name).unwrap(), 1, value.as_ptr());
        }
    }

    /// Resolve the property name to a memory-location
    fn get_uniform_location(&self, property_name: &str) -> Result<GLint, String> {
        let name = std::ffi::CString::new(property_name)
            .map_err(|_| "Could not load CString")?;
        unsafe {
            Ok(gl::GetUniformLocation(self.id, name.as_bytes_with_nul().as_ptr().cast()))
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        info!(target: LOG_TARGET, "Deleting program {}", self.id);
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}
