use std::sync::Arc;

use bevy::{
    math::Vec2,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::Image,
    },
};
use fdsm::{
    bezier::{scanline::FillRule, Point, Segment},
    shape::{Contour, Shape},
    transform::Transform,
};
use image::{GrayImage, RgbaImage};
use nalgebra::{Affine2, Similarity2, Vector2};
use owned_ttf_parser::{AsFaceRef, OutlineBuilder, Rect};

pub use owned_ttf_parser::GlyphId;

#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub id: GlyphId,
    pub advance: Vec2,
    pub offset: Vec2,
    pub size: Vec2,
}

#[derive(Asset, TypePath, Clone)]
pub struct Font {
    // TODO: parse on-demand instead of storing a owned ttf-Face?
    face: Arc<owned_ttf_parser::OwnedFace>,
}

impl Font {
    pub fn from(face: owned_ttf_parser::OwnedFace) -> Self {
        Self {
            face: Arc::new(face),
        }
    }

    pub fn glyph(&self, code_point: char) -> Option<GlyphInfo> {
        let face = self.face.clone();
        let face = face.as_ref().as_face_ref();

        let id = face.glyph_index(code_point)?;

        let bounds = face.glyph_bounding_box(id).unwrap_or(Rect {
            x_min: 0,
            y_min: 0,
            x_max: 0,
            y_max: 0,
        });
        let scale = 1f32 / face.units_per_em() as f32;

        Some(GlyphInfo {
            id,
            advance: Vec2::new(
                face.glyph_hor_advance(id).unwrap_or_default() as f32,
                face.glyph_ver_advance(id).unwrap_or_default() as f32,
            ) * scale,
            offset: Vec2::new(bounds.x_min as f32, bounds.y_min as f32) * scale,
            size: Vec2::new(
                (bounds.x_max - bounds.x_min) as f32,
                (bounds.y_max - bounds.y_min) as f32,
            ) * scale,
        })
    }

    fn load_from_face(
        face: &owned_ttf_parser::Face,
        glyph_id: GlyphId,
    ) -> fdsm::shape::Shape<fdsm::shape::Contour> {
        let mut builder = ShapeBuilder {
            shape: Shape::default(),
            start_point: None,
            last_point: None,
        };
        face.outline_glyph(glyph_id, &mut builder);
        builder.shape
    }

    pub fn generate(&self, glyph_id: GlyphId, range: f64) -> Option<Image> {
        let face = self.face.clone();
        let face = face.as_ref().as_face_ref();

        // TODO: don't hard code the scale..
        let scale = (1.0f64 / face.units_per_em() as f64) * 100f64;

        let bbox = face.glyph_bounding_box(glyph_id)?;
        let transformation = nalgebra::convert::<_, Affine2<f64>>(Similarity2::new(
            Vector2::new(
                range - bbox.x_min as f64 * scale,
                range - bbox.y_min as f64 * scale,
            ),
            0.0,
            scale,
        ));
        let mut shape = Self::load_from_face(face, glyph_id);
        shape.transform(&transformation);

        let width = ((bbox.x_max as f64 - bbox.x_min as f64) * scale + range * 2f64).ceil() as u32;
        let height = ((bbox.y_max as f64 - bbox.y_min as f64) * scale + range * 2f64).ceil() as u32;

        // let colored_shape = Shape::edge_coloring_simple(shape, 0.01f64, 69420);
        // let prepared_colored_shape = colored_shape.prepare();
        // let mut msdf = RgbImage::new(width, height);
        // fdsm::generate::generate_msdf(&prepared_colored_shape, range, &mut msdf);
        // fdsm::render::correct_sign_msdf(&mut msdf, &prepared_colored_shape, FillRule::Nonzero);

        // let mut msdf_rgba = RgbaImage::new(width, height);
        // for (output, chunk) in msdf_rgba.chunks_exact_mut(4).zip(msdf.chunks_exact(3)) {
        //     output.copy_from_slice(&[chunk[0], chunk[1], chunk[2], 0]);
        // }

        let prepared_shape = shape.prepare();
        let mut sdf = GrayImage::new(width, height);
        fdsm::generate::generate_sdf(&prepared_shape, range, &mut sdf);
        fdsm::render::correct_sign_sdf(&mut sdf, &prepared_shape, FillRule::Nonzero);

        let mut msdf_rgba = RgbaImage::new(width, height);
        for (output, luma) in msdf_rgba.chunks_exact_mut(4).zip(sdf.iter()) {
            output.copy_from_slice(&[0, 0, 0, *luma]);
        }

        Some(Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            msdf_rgba.into_raw(),
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::MAIN_WORLD,
        ))
    }

    pub fn line_gap(&self) -> f64 {
        let face = self.face.clone();
        let face = face.as_ref().as_face_ref();
        face.height() as f64 / face.units_per_em() as f64
    }
}

// stolen from fdsm ttf-importer

#[derive(Debug)]
struct ShapeBuilder {
    shape: Shape<Contour>,
    start_point: Option<Point>,
    last_point: Option<Point>,
}

impl OutlineBuilder for ShapeBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        // eprintln!("move_to {x} {y}");
        if let Some(contour) = self.shape.contours.last_mut() {
            if self.start_point != self.last_point {
                contour.segments.push(Segment::line(
                    self.last_point.unwrap(),
                    self.start_point.unwrap(),
                ));
            }
        }
        self.start_point = Some(Point::new(x.into(), y.into()));
        self.last_point = self.start_point;
        self.shape.contours.push(Contour::default());
    }

    fn line_to(&mut self, x: f32, y: f32) {
        // eprintln!("line_to {x} {y}");
        let next_point = Point::new(x.into(), y.into());
        self.shape
            .contours
            .last_mut()
            .unwrap()
            .segments
            .push(Segment::line(self.last_point.unwrap(), next_point));
        self.last_point = Some(next_point);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        // eprintln!("quad_to {x1} {y1} {x} {y}");
        let next_point = Point::new(x.into(), y.into());
        self.shape
            .contours
            .last_mut()
            .unwrap()
            .segments
            .push(Segment::quad(
                self.last_point.unwrap(),
                Point::new(x1.into(), y1.into()),
                next_point,
            ));
        self.last_point = Some(next_point);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        // eprintln!("curve_to {x1} {y1} {x2} {y2} {x} {y}");
        let next_point = Point::new(x.into(), y.into());
        self.shape
            .contours
            .last_mut()
            .unwrap()
            .segments
            .push(Segment::cubic(
                self.last_point.unwrap(),
                Point::new(x1.into(), y1.into()),
                Point::new(x2.into(), y2.into()),
                next_point,
            ));
        self.last_point = Some(next_point);
    }

    fn close(&mut self) {
        if let Some(contour) = self.shape.contours.last_mut() {
            if self.start_point != self.last_point {
                contour.segments.push(Segment::line(
                    self.last_point.take().unwrap(),
                    self.start_point.take().unwrap(),
                ));
            }
        }
    }
}
