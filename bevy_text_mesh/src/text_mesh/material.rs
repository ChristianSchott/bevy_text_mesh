use bevy::{
    asset::{Asset, AssetPath, Handle},
    prelude::{AlphaMode, Image, *},
    reflect::TypePath,
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{AsBindGroup, ShaderRef, VertexFormat},
    },
};

pub const ATTRIBUTE_TEXT_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("Text_Position", 988540917, VertexFormat::Float32x2);

pub const SDF_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(98131239812464981);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SdfMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub sdf_texture: Handle<Image>,
}

impl Material for SdfMaterial {
    fn vertex_shader() -> ShaderRef {
        SDF_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        SDF_SHADER_HANDLE.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    // https://bevyengine.org/examples/shaders/custom-vertex-attribute/
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        // TODO: store position/uv/color per char in SBO, instead of per vertex
        let vertex_layout = layout.0.get_layout(&[
            ATTRIBUTE_TEXT_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(2),
        ])?;

        descriptor.vertex.buffers = vec![vertex_layout];
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}
