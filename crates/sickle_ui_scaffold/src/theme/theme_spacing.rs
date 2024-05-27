use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct DividerSpacing {
    pub extra_small: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub custom_1: f32,
    pub custom_2: f32,
}

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct Spacing {
    pub tiny: f32,
    pub extra_small: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub extra_large: f32,
    pub custom_1: f32,
    pub custom_2: f32,
    pub custom_3: f32,
    pub custom_4: f32,
}

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct IconSizes {
    pub extra_small: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub custom_1: f32,
    pub custom_2: f32,
}

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct CheckboxSizes {
    pub line_height: f32,
    pub border_size: f32,
    pub checkbox_size: f32,
    pub checkmark_size: f32,
}

impl CheckboxSizes {
    pub fn line_padding(&self) -> f32 {
        (self.line_height - self.checkbox_size - 2. * self.border_size) / 2.
    }

    pub fn checkbox_size(&self) -> f32 {
        self.checkbox_size + 2. * self.border_size
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct InputSizes {
    pub checkbox: CheckboxSizes,
}

// TODO: bevy 0.14: Add shape / border radius
#[derive(Clone, Copy, Debug, Reflect)]
pub struct ThemeSpacing {
    pub borders: DividerSpacing,
    pub gaps: Spacing,
    pub areas: Spacing,
    pub icons: IconSizes,
    pub inputs: InputSizes,
}

impl Default for ThemeSpacing {
    fn default() -> Self {
        Self {
            borders: DividerSpacing {
                extra_small: 1.,
                small: 2.,
                medium: 4.,
                large: 8.,
                custom_1: 3.,
                custom_2: 6.,
            },
            gaps: Spacing {
                tiny: 1.,
                extra_small: 2.,
                small: 4.,
                medium: 8.,
                large: 16.,
                extra_large: 32.,
                custom_1: 2.,
                custom_2: 6.,
                custom_3: 22.,
                custom_4: 48.,
            },
            areas: Spacing {
                tiny: 8.,
                extra_small: 16.,
                small: 24.,
                medium: 32.,
                large: 64.,
                extra_large: 128.,
                custom_1: 12.,
                custom_2: 36.,
                custom_3: 48.,
                custom_4: 96.,
            },
            icons: IconSizes {
                extra_small: 12.,
                small: 16.,
                medium: 24.,
                large: 32.,
                custom_1: 48.,
                custom_2: 64.,
            },
            inputs: InputSizes {
                checkbox: CheckboxSizes {
                    line_height: 32.,
                    border_size: 1.,
                    checkbox_size: 14.,
                    checkmark_size: 14.,
                },
            },
        }
    }
}
