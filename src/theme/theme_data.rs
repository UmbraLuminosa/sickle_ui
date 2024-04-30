use bevy::prelude::*;

use super::{
    theme_colors::{SchemeColors, ThemeColors},
    theme_spacing::ThemeSpacing,
    typography::ThemeTypography,
};

#[derive(Resource, Clone, Copy, Debug, Default, Reflect)]
pub enum Contrast {
    #[default]
    Standard,
    Medium,
    High,
}

#[derive(Resource, Clone, Copy, Debug, Reflect)]
pub enum Scheme {
    Light(Contrast),
    Dark(Contrast),
}

impl Default for Scheme {
    fn default() -> Self {
        Self::Light(Default::default())
    }
}

impl Scheme {
    pub fn is_light(&self) -> bool {
        matches!(self, Scheme::Light(_))
    }

    pub fn is_dark(&self) -> bool {
        matches!(self, Scheme::Dark(_))
    }
}

#[derive(Resource, Clone, Debug, Default, Reflect)]
pub struct ThemeData {
    pub active_scheme: Scheme,
    pub colors: ThemeColors,
    pub spacing: ThemeSpacing,
    pub text: ThemeTypography,
    // Colors, floats, bools, strings (image/font path), handles
    // font data-> text styles -> per weight
    // should act as a cache for handles
}

impl ThemeData {
    /// Returns the scheme colors of the current active scheme / contrast
    pub fn colors(&self) -> SchemeColors {
        match self.active_scheme {
            Scheme::Light(contrast) => self.colors.schemes.light.contrast(contrast),
            Scheme::Dark(contrast) => self.colors.schemes.dark.contrast(contrast),
        }
    }
}
