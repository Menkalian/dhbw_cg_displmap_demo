use crate::glhelper::{Program, TextureData};
use crate::glhelper::utils::format_texture_path;
use crate::Resources;

/// # TextureCollection
/// A collection of textures with different purposes for the same content.
/// The following types of textures are contained within a collection:
///  - Basic texture (color/image)
///  - Normal Map
///  - Height Map
pub struct TextureCollection {
    base: TextureData,
    normal: TextureData,
    height: TextureData,
}

impl TextureCollection {
    /// # Static utility
    /// Configure the given program to assign the textures to the correct samplers
    pub fn configure_program(program: &Program) {
        program.set_property_int("baseMap", 0);
        program.set_property_int("normalMap", 1);
        program.set_property_int("heightMap", 2);
    }

    /// # Constructor
    /// Load the given texture collection from the resources
    pub fn from_resources(res: &Resources, texture_name: &str, extension: &str) -> Result<TextureCollection, String> {
        let base = TextureData::from_resources(&res, format_texture_path(texture_name, "base", extension).as_str()).unwrap();
        let normal = TextureData::from_resources(&res, format_texture_path(texture_name, "normal", extension).as_str()).unwrap();
        let height = TextureData::from_resources(&res, format_texture_path(texture_name, "height", extension).as_str()).unwrap();

        Ok(TextureCollection {
            base,
            normal,
            height,
        })
    }

    /// Loads all textures from the collection into the shader
    pub fn set_active(&self) {
        self.base.bind_texture(gl::TEXTURE0);
        self.normal.bind_texture(gl::TEXTURE1);
        self.height.bind_texture(gl::TEXTURE2);
    }
}
