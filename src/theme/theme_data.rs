use bevy::prelude::*;
use sickle_math::ease::Ease;

use crate::ui_style::StyleBuilder;

use super::{
    icons::Icons,
    theme_colors::{SchemeColors, ThemeColors},
    theme_spacing::ThemeSpacing,
    typography::ThemeTypography,
    AnimationSettings, DynamicStyle, UiContext,
};

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub enum Contrast {
    #[default]
    Standard,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Reflect)]
pub enum Scheme {
    Light(Contrast),
    Dark(Contrast),
}

impl Default for Scheme {
    fn default() -> Self {
        Self::Dark(Default::default())
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

#[derive(Resource, Clone, Debug, Reflect)]
pub struct ThemeData {
    pub active_scheme: Scheme,
    pub colors: ThemeColors,
    pub spacing: ThemeSpacing,
    pub text: ThemeTypography,
    pub icons: Icons,
    pub interaction_animation: AnimationSettings,
    pub enter_animation: AnimationSettings,
}

impl Default for ThemeData {
    fn default() -> Self {
        let mut interaction_animation = AnimationSettings::new();
        interaction_animation
            .pointer_enter(0.1, Ease::OutExpo, None)
            .pointer_leave(0.1, Ease::OutExpo, None)
            .press(0.1, Ease::OutExpo, None);

        let mut enter_animation = AnimationSettings::new();
        enter_animation.enter(0.1, Ease::OutExpo, None);

        Self {
            active_scheme: Default::default(),
            colors: Default::default(),
            spacing: Default::default(),
            text: Default::default(),
            icons: Default::default(),
            interaction_animation,
            enter_animation,
        }
    }
}

impl ThemeData {
    pub fn with_default(builder: fn(&mut StyleBuilder, &ThemeData)) -> StyleBuilder {
        let theme_data = ThemeData::default();
        let mut style_builder = StyleBuilder::new();
        builder(&mut style_builder, &theme_data);

        style_builder
    }

    pub fn with_default_and_override(
        builder: fn(&mut StyleBuilder, &ThemeData),
        context: &impl UiContext,
        style_override: impl FnOnce(&mut StyleBuilder),
    ) -> DynamicStyle {
        let mut override_style_builder = StyleBuilder::new();
        style_override(&mut override_style_builder);
        let override_style = override_style_builder.convert_with(context);
        ThemeData::with_default(builder)
            .convert_with(context)
            .merge(override_style)
    }

    /// Returns the scheme colors of the current active scheme / contrast
    pub fn colors(&self) -> SchemeColors {
        match self.active_scheme {
            Scheme::Light(contrast) => self.colors.schemes.light.contrast(contrast),
            Scheme::Dark(contrast) => self.colors.schemes.dark.contrast(contrast),
        }
    }
}
