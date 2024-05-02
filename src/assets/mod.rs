//! Adds built-in `sickle_ui` assets via [`embedding`](https://docs.rs/bevy/latest/bevy/asset/macro.embedded_asset.html).
//!
//! Access these assets with `asset_server.load("embedded://sickle_ui/icons/exit.png")`.
//!

mod fonts;
mod icons;

use bevy::prelude::*;

use fonts::BuiltInFontsPlugin;
use icons::BuiltInIconsPlugin;

pub(crate) struct BuiltInAssetsPlugin;

impl Plugin for BuiltInAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BuiltInFontsPlugin, BuiltInIconsPlugin));
    }
}
