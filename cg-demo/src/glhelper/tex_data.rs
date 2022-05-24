use gl::types::{GLenum, GLuint};
use log::{debug, info, trace};

use crate::glhelper::utils::load_texture_from_path;
use crate::Resources;

const LOG_TARGET: &str = "Texture";

/// # TextureData
/// Handle for a texture loaded in OpenGL
pub struct TextureData {
    id: GLuint,
}

impl TextureData {
    /// # Constructor
    /// Load the given texture from the resources
    pub fn from_resources(res: &Resources, res_name: &str) -> Result<TextureData, String> {
        let tex_id = load_texture_from_path(
            res.construct_path(res_name)
                .map_err(|_| "Could not create path to resource")?.as_path()
        ).map_err(|e| e)?;
        info!(target: LOG_TARGET, "Loaded texture \"{}\" from resources as texture {}", res_name, tex_id);

        Ok(TextureData { id: tex_id })
    }

    /// Bind the associated texture to the given texture_unit
    pub fn bind_texture(&self, texture_unit: GLenum) {
        trace!(target: LOG_TARGET, "Using texture {} for unit {}", self.id, texture_unit - gl::TEXTURE0);
        unsafe {
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    /// Get the texture id in OpenGL
    pub fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for TextureData {
    fn drop(&mut self) {
        info!(target: LOG_TARGET, "Unloading texture {}", self.id);
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
