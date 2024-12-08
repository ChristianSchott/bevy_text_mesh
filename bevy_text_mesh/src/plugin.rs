use bevy::app::Plugin;
use bevy::prelude::*;

use super::font::SdfFontPlugin;
use super::text_mesh::TextMeshPlugin;

pub struct Text3dPlugin;

impl Plugin for Text3dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SdfFontPlugin).add_plugins(TextMeshPlugin);
    }
}
