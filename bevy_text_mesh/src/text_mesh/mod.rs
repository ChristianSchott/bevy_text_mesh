use bevy::app::PostUpdate;
use bevy::asset::load_internal_asset;
use bevy::prelude::{App, IntoSystemConfigs, Shader};
use bevy::{app::Plugin, pbr::MaterialPlugin};
use material::SdfMaterial;

mod material;
mod text_mesh;

pub use super::font::Font;
pub use super::font::FontAtlases;
pub use text_mesh::Glyph;
pub use text_mesh::TextMesh;

pub struct TextMeshPlugin;

impl Plugin for TextMeshPlugin {
    fn build(&self, app: &mut App) {
        // embed the SDF shader in the binary
        load_internal_asset!(
            app,
            material::SDF_SHADER_HANDLE,
            "shaders/sdf.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<SdfMaterial>::default())
            .add_systems(
                PostUpdate,
                (
                    text_mesh::update_font_atlases,
                    text_mesh::create_atlas_meshes,
                    text_mesh::update_text_mesh,
                )
                    .chain(),
            );
    }
}
