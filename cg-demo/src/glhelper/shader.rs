use std::ffi::CStr;

use gl::types::{GLenum, GLuint};
use log::info;

use crate::glhelper::utils::compile_shader_from_source;
use crate::Resources;

const LOG_TARGET: &str = "Shader";

/// # Shader
/// Handle for an OpenGL-Shader
pub struct Shader {
    id: GLuint,
}

impl Shader {
    /// # Constructor
    /// Compiles the given shader source code
    pub fn from_source(
        souce: &CStr,
        kind: GLenum,
    ) -> Result<Shader, String> {
        let id = compile_shader_from_source(souce, kind)?;
        info!("Compiled new shader {}", id);
        Ok(Shader { id })
    }

    /// # Constructor
    /// Compiles the given shader from the resources
    pub fn from_res(res: &Resources, name: &str) -> Result<Shader, String> {
        // Possible Shader extensions to look for
        const POSSIBLE_EXT: [(&str, GLenum); 2] = [
            (".vert", gl::VERTEX_SHADER),
            (".frag", gl::FRAGMENT_SHADER),
        ];

        let shader_kind = POSSIBLE_EXT.iter()
            .find(|&&(file_extension, _)| {
                name.ends_with(file_extension)
            })
            .map(|&(_, kind)| kind)
            .ok_or_else(|| format!("Can not determine shader type for resource {}", name))?;

        info!("Compiling shader \"{}\" as {:?}", name, shader_kind);
        let source = res.load_cstring(name)
            .map_err(|e| format!("Error loading resource {}: {:?}", name, e))?;

        Shader::from_source(&source, shader_kind)
    }

    /// Get id of the shader
    pub fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        info!(target: LOG_TARGET, "Deleting shader {}", self.id);
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}


