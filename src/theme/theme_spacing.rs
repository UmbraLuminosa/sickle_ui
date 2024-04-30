use bevy::prelude::*;

#[derive(Clone, Debug, Default, Reflect)]
pub struct DividerSpacing {
    extra_small: f32,
    small: f32,
    medium: f32,
    large: f32,
    custom_1: f32,
    custom_2: f32,
}

#[derive(Clone, Debug, Default, Reflect)]
pub struct Spacing {
    tiny: f32,
    extra_small: f32,
    small: f32,
    medium: f32,
    large: f32,
    extra_large: f32,
    custom_1: f32,
    custom_2: f32,
    custom_3: f32,
    custom_4: f32,
}

#[derive(Clone, Debug, Reflect)]
pub struct ThemeSpacing {
    borders: DividerSpacing,
    gaps: Spacing,
    areas: Spacing,
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
                tiny: 4.,
                extra_small: 8.,
                small: 12.,
                medium: 16.,
                large: 32.,
                extra_large: 64.,
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
        }
    }
}
