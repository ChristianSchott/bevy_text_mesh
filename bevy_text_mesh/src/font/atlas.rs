use bevy::{
    asset::{AssetId, Assets, Handle},
    log,
    math::{Rect, UVec2},
    prelude::Resource,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::Image,
    },
    sprite::{DynamicTextureAtlasBuilder, TextureAtlasLayout},
    utils::{HashMap, HashSet},
};

use super::font::{Font, GlyphId, GlyphInfo};

const ATLAS_BASE_SIZE: u32 = 1024;

pub struct FontAtlas {
    pub dynamic_texture_atlas_builder: DynamicTextureAtlasBuilder,
    pub glyph_to_atlas_index: HashMap<GlyphId, usize>,
    pub texture_atlas: TextureAtlasLayout,
    pub texture: Handle<Image>,
}

impl FontAtlas {
    pub fn new(
        textures: &mut Assets<Image>,
        // materials: &mut Assets<SdfMaterial>,
        size: UVec2,
    ) -> FontAtlas {
        let texture = textures.add(Image::new_fill(
            Extent3d {
                width: size.x as u32,
                height: size.y as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
            // Need to keep this image CPU persistent in order to add additional glyphs later on
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        ));
        Self {
            texture_atlas: TextureAtlasLayout::new_empty(size),
            glyph_to_atlas_index: HashMap::default(),
            dynamic_texture_atlas_builder: DynamicTextureAtlasBuilder::new(size, 1),
            texture,
        }
    }

    pub fn add_glyph(
        &mut self,
        textures: &mut Assets<Image>,
        glyph_id: GlyphId,
        texture: &Image,
    ) -> bool {
        if let Some(index) = self.dynamic_texture_atlas_builder.add_texture(
            &mut self.texture_atlas,
            textures,
            &texture,
            &self.texture,
        ) {
            self.glyph_to_atlas_index.insert(glyph_id, index);
            true
        } else {
            false
        }
    }

    pub fn lookup_glyph(&self, glyph_id: GlyphId, range: u8) -> Option<Rect> {
        self.glyph_to_atlas_index
            .get(&glyph_id)
            .and_then(|index| self.texture_atlas.textures.get(*index))
            .map(|rect| {
                let size_inv = 1f32 / self.texture_atlas.size.as_vec2();
                let rect = rect.inflate(-(range as i32)).as_rect();
                Rect::from_corners(rect.min * size_inv, rect.max * size_inv)
            })
    }
}

pub struct FontData {
    atlases: Vec<FontAtlas>,
    added: HashSet<char>,
    code_point_to_atlas: HashMap<char, usize>,
    code_point_to_glyph_info: HashMap<char, GlyphInfo>,
    range: u8,
    line_gap: f64,
}

impl FontData {
    pub fn from(face: &Font) -> Self {
        Self {
            atlases: vec![],
            added: Default::default(),
            code_point_to_atlas: Default::default(),
            code_point_to_glyph_info: Default::default(),
            range: 6,
            line_gap: face.line_gap(),
        }
    }

    pub fn has_glyph(&self, code_point: char) -> bool {
        self.added.contains(&code_point)
    }

    pub fn add_glyph(
        &mut self,
        code_point: char,
        font: &Font,
        textures: &mut Assets<Image>,
    ) -> Option<usize> {
        self.added.insert(code_point);
        let Some(glyph_info) = font.glyph(code_point) else {
            log::warn!("No glyph generated for {code_point}. No glyph data available");
            return None;
        };
        self.code_point_to_glyph_info
            .insert(code_point, glyph_info.clone());
        let glyph_texture = font.generate(glyph_info.id, self.range as f64)?;
        let atlas_index = self
            .atlases
            .iter_mut()
            .enumerate()
            .find_map(|(index, atlas)| {
                // find a texture atlas with enough space to hold the glyph
                atlas
                    .add_glyph(textures, glyph_info.id, &glyph_texture)
                    .then_some(index)
            })
            .unwrap_or_else(|| {
                // otherwise create a new texture atlas
                // Pick the higher of 1024 or the smallest power of 2 greater than glyph_max_size
                let glyph_max_size: u32 = glyph_texture.width().max(glyph_texture.height());
                let containing =
                    (1u32 << (32 - glyph_max_size.leading_zeros())).max(ATLAS_BASE_SIZE);
                let mut atlas = FontAtlas::new(textures, UVec2::new(containing, containing));
                if !atlas.add_glyph(textures, glyph_info.id, &glyph_texture) {
                    log::error!("Failed adding glyph!");
                }
                let idx = self.atlases.len();
                self.atlases.push(atlas);
                idx
            });
        self.code_point_to_atlas.insert(code_point, atlas_index);
        Some(atlas_index)
    }

    pub fn glyph_info(&self, code_point: char) -> Option<&GlyphInfo> {
        self.code_point_to_glyph_info.get(&code_point)
    }

    pub fn atlas_count(&self) -> usize {
        self.atlases.len()
    }

    pub fn atlas(&self, code_point: char) -> Option<usize> {
        self.code_point_to_atlas.get(&code_point).copied()
    }

    pub fn lookup_glyph(&self, glyph_id: GlyphId) -> Option<Rect> {
        self.atlases
            .iter()
            .find_map(|atlas| atlas.lookup_glyph(glyph_id, self.range))
    }

    pub fn atlas_texture(&self, atlas: usize) -> Option<Handle<Image>> {
        self.atlases
            .get(atlas)
            .map(|font_atlas| font_atlas.texture.clone())
    }

    pub fn line_gap(&self) -> f32 {
        self.line_gap as f32
    }
}

#[derive(Default, Resource)]
pub struct FontAtlases {
    font_data: HashMap<AssetId<Font>, FontData>,
}

impl FontAtlases {
    pub fn add_code_points<'c>(
        &mut self,
        chars: &[char],
        font_id: AssetId<Font>,
        fonts: &Assets<Font>,
        textures: &mut Assets<Image>,
        // materials: &mut Assets<SdfMaterial>,
    ) {
        let Some(font) = fonts.get(font_id) else {
            bevy::log::error!("Font not found!");
            return;
        };
        let font_data = self.font_data.entry(font_id).or_insert_with(|| {
            bevy::log::info!("Inserting new FontData entry.");
            FontData::from(font)
        });
        for c in chars {
            if !font_data.has_glyph(*c) {
                if let Some(i) = font_data.add_glyph(*c, font, textures) {
                    bevy::log::info!("Code point {c} added to {i}!");
                }
            }
        }
    }

    pub fn data(&self, font_id: AssetId<Font>) -> Option<&FontData> {
        self.font_data.get(&font_id)
    }
}
