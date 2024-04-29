use bevy::prelude::*;

use super::theme_colors::ThemeColors;

#[derive(Resource, Clone, Debug, Reflect)]
pub struct ThemeData {
    pub theme_colors: ThemeColors,
    pub background_color: Color,
    // Colors, floats, bools, strings (image/font path), handles
    // font data-> text styles -> per weight
    // should act as a cache for handles
}

impl Default for ThemeData {
    fn default() -> Self {
        Self {
            theme_colors: ThemeColors::default(),
            background_color: Color::BLUE,
        }
    }
}
