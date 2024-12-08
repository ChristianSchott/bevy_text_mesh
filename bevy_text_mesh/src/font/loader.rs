use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
};
use std::{future::Future, pin::Pin};
use thiserror::Error;

use super::font::Font;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum FontLoaderError {
    /// An [IO](std::io) Error
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// An [InvalidFont](ab_glyph::InvalidFont) Error
    #[error(transparent)]
    FontInvalid(#[from] owned_ttf_parser::FaceParsingError),
}

#[derive(Default)]
pub struct FontLoader;

impl AssetLoader for FontLoader {
    type Asset = Font;
    type Settings = ();
    type Error = FontLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> Pin<Box<dyn Future<Output = Result<Font, Self::Error>> + Send + 'a>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            bevy::asset::AsyncReadExt::read_to_end(reader, &mut bytes).await?;
            let face = owned_ttf_parser::OwnedFace::from_vec(bytes, 0)?;
            Ok(Font::from(face))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ttf", "otf"]
    }
}
