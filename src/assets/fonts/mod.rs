use bevy::{asset::embedded_asset, prelude::*};

pub(crate) struct BuiltInFontsPlugin;

impl Plugin for BuiltInFontsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "src/assets/", "FiraSans-Bold.ttf");
        embedded_asset!(app, "src/assets/", "FiraSans-BoldItalic.ttf");
        embedded_asset!(app, "src/assets/", "FiraSans-Italic.ttf");
        embedded_asset!(app, "src/assets/", "FiraSans-Medium.ttf");
        embedded_asset!(app, "src/assets/", "FiraSans-MediumItalic.ttf");
        embedded_asset!(app, "src/assets/", "FiraSans-Regular.ttf");
        embedded_asset!(app, "src/assets/", "FiraSansCondensed-Bold.ttf");
        embedded_asset!(app, "src/assets/", "FiraSansCondensed-BoldItalic.ttf");
        embedded_asset!(app, "src/assets/", "FiraSansCondensed-Italic.ttf");
        embedded_asset!(app, "src/assets/", "FiraSansCondensed-Regular.ttf");
        embedded_asset!(app, "src/assets/", "MaterialIcons-Regular.ttf");
    }
}
