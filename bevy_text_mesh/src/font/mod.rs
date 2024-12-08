use bevy::{
    app::{App, Plugin},
    asset::AssetApp,
};
use loader::FontLoader;

mod atlas;
mod font;
mod loader;

pub use atlas::FontAtlases;
pub use atlas::FontData;
pub use font::Font;

pub struct SdfFontPlugin;

impl Plugin for SdfFontPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Font>()
            .init_asset_loader::<FontLoader>()
            .init_resource::<FontAtlases>();
    }
}
