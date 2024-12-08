use bevy::asset::{AssetId, Assets};
use bevy::math::Rect;
use bevy::pbr::MaterialMeshBundle;
use bevy::prelude::{BuildChildren, Changed, Commands, Entity, Image, Query, Res, ResMut};
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::utils::HashMap;
use bevy::{
    asset::Handle,
    prelude::{Component, Mesh},
};

use super::material::{SdfMaterial, ATTRIBUTE_TEXT_POSITION};
use super::{Font, FontAtlases};

pub struct PositionedGlyph {
    pub position: Rect,
    pub uv: Rect,
    pub index: usize,
    pub color: [f32; 4],
}

pub struct Glyph {
    pub position: Rect,
    pub character: char,
    pub color: [f32; 4],
}

#[derive(Component)]
pub struct TextMesh {
    font: Handle<Font>,
    missing: Vec<char>,
    glyphs: Box<[Glyph]>,
    meshes: HashMap<usize, Handle<Mesh>>,
    child_entities: HashMap<usize, Entity>,
}

impl TextMesh {
    pub fn new(font: Handle<Font>) -> Self {
        Self {
            font,
            missing: Default::default(),
            glyphs: Default::default(),
            meshes: Default::default(),
            child_entities: Default::default(),
        }
    }

    pub fn font_id(&self) -> AssetId<Font> {
        self.font.id()
    }

    pub fn add_missing(&mut self, missing: &[char]) {
        self.missing.extend_from_slice(missing);
    }

    pub fn set_glyphs(&mut self, glyphs: Box<[Glyph]>) {
        self.glyphs = glyphs;
    }
}

pub fn update_font_atlases(
    mut query: Query<&mut TextMesh, Changed<TextMesh>>,
    mut atlases: ResMut<FontAtlases>,
    mut textures: ResMut<Assets<Image>>,
    fonts: Res<Assets<Font>>,
) {
    for mut text_mesh in query.iter_mut() {
        atlases.add_code_points(
            &text_mesh.missing,
            text_mesh.font_id(),
            &fonts,
            &mut textures,
        );
        text_mesh.missing.clear();
    }
}

pub fn create_atlas_meshes(
    mut query: Query<(Entity, &mut TextMesh), Changed<TextMesh>>,
    mut commands: Commands,
    font_atlas: Res<FontAtlases>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SdfMaterial>>,
) {
    for (entity, mut text_mesh) in query.iter_mut() {
        if let Some(data) = font_atlas.data(text_mesh.font.id()) {
            // TODO only create necessary atlas meshes
            for i in 0..data.atlas_count() {
                if text_mesh.meshes.contains_key(&i) {
                    continue;
                }

                let mesh = meshes.add(Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                ));
                text_mesh.meshes.insert(i, mesh.clone());
                let child = commands
                    .spawn((MaterialMeshBundle {
                        mesh: mesh,
                        material: materials.add(SdfMaterial {
                            sdf_texture: data.atlas_texture(i).unwrap(),
                        }),
                        ..Default::default()
                    },))
                    .set_parent(entity)
                    .id();
                text_mesh.child_entities.insert(i, child);
            }
        }
    }
}

pub fn update_text_mesh(
    mut query: Query<&TextMesh, Changed<TextMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    font_atlas: Res<FontAtlases>,
) {
    for text_mesh in query.iter_mut() {
        // bevy::log::info!("Regenerate");
        let Some(data) = font_atlas.data(text_mesh.font.id()) else {
            continue;
        };

        for (index, mesh) in text_mesh.meshes.iter() {
            if let Some(mesh) = meshes.get_mut(mesh) {
                let mut builder = TextMeshBuilder::new(mesh);
                for glyph in text_mesh.glyphs.iter().filter(|glyph| {
                    data.atlas(glyph.character)
                        .map(|atlas| atlas == *index)
                        .unwrap_or(false)
                }) {
                    if let Some(atlas_rect) = data
                        .glyph_info(glyph.character)
                        .and_then(|info| data.lookup_glyph(info.id))
                    {
                        builder.append_glyph(&glyph.position, &atlas_rect, &glyph.color);
                    }
                }
            }
        }
    }
}

struct TextMeshBuilder<'a> {
    index: u32,
    mesh: &'a mut Mesh,
}

// if we want to move text Z-direction relative to the other text, we may need f32x3 here..
// pub const ATTRIBUTE_TEXT_POSITION: MeshVertexAttribute =
//     MeshVertexAttribute::new("Text_Position", 988540917, VertexFormat::Float32x2);

impl<'a> TextMeshBuilder<'a> {
    fn new(mesh: &'a mut Mesh) -> Self {
        if !mesh.contains_attribute(ATTRIBUTE_TEXT_POSITION) {
            mesh.insert_attribute(
                ATTRIBUTE_TEXT_POSITION,
                VertexAttributeValues::Float32x2(vec![]),
            );
        }
        if !mesh.contains_attribute(Mesh::ATTRIBUTE_UV_0) {
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_UV_0,
                VertexAttributeValues::Float32x2(vec![]),
            );
        }
        // FIXME: 4 vertices with f32x4 for color seems overkill for a single color glyph
        if !mesh.contains_attribute(Mesh::ATTRIBUTE_COLOR) {
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_COLOR,
                VertexAttributeValues::Float32x4(vec![]),
            );
        }
        if !mesh.indices().is_some() {
            mesh.insert_indices(Indices::U32(vec![]));
        }

        if let Some(VertexAttributeValues::Float32x2(vertices)) =
            mesh.attribute_mut(ATTRIBUTE_TEXT_POSITION)
        {
            vertices.clear();
        }
        if let Some(VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            uvs.clear();
        }
        if let Some(VertexAttributeValues::Float32x4(colors)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
        {
            colors.clear();
        }
        if let Some(Indices::U32(indices)) = mesh.indices_mut() {
            indices.clear();
        }

        Self { index: 0, mesh }
    }

    fn append_glyph(&mut self, position: &Rect, uv: &Rect, color: &[f32; 4]) {
        if let Some(VertexAttributeValues::Float32x2(vertices)) =
            self.mesh.attribute_mut(ATTRIBUTE_TEXT_POSITION)
        {
            let rect = *position;
            vertices.push([rect.min.x, rect.min.y]);
            vertices.push([rect.max.x, rect.min.y]);
            vertices.push([rect.max.x, rect.max.y]);
            vertices.push([rect.min.x, rect.max.y]);
        }

        if let Some(VertexAttributeValues::Float32x2(uvs)) =
            self.mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            let rect = *uv;
            uvs.push([rect.min.x, rect.min.y]);
            uvs.push([rect.max.x, rect.min.y]);
            uvs.push([rect.max.x, rect.max.y]);
            uvs.push([rect.min.x, rect.max.y]);
        }

        if let Some(VertexAttributeValues::Float32x4(colors)) =
            self.mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
        {
            colors.extend([*color; 4]); // FIXME: this wastes a ton of memory..
        }

        if let Some(Indices::U32(indices)) = self.mesh.indices_mut() {
            let base = self.index * 4;
            indices.extend([base + 0, base + 1, base + 3, base + 1, base + 2, base + 3]);
        }

        self.index += 1;
    }
}
