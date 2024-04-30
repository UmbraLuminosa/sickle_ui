use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum FontStyle {
    Display,
    Headline,
    Title,
    Body,
    Label,
}

#[derive(Clone, Copy, Debug)]
pub enum FontScale {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug)]
pub enum FontType {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

#[derive(Clone, Debug, Default, Reflect)]
pub struct FontSet {
    pub regular: String,
    pub bold: String,
    pub italic: String,
    pub bold_italic: String,
}

#[derive(Clone, Debug, Default, Reflect)]
pub struct FontConfig {
    pub font: FontSet,
    // Unusued until proper text handling exists
    pub weight: f32,
    // Unusued until proper text handling exists
    //pub weight_prominent: Option<f32>,
    // Unusued until proper text handling exists
    pub tracking: f32,

    pub size: f32,
    pub line_height: f32,
}

#[derive(Clone, Debug, Default, Reflect)]
pub struct StyleScales {
    pub small: FontConfig,
    pub medium: FontConfig,
    pub large: FontConfig,
}

#[derive(Clone, Debug, Reflect)]
pub struct ThemeTypography {
    pub display: StyleScales,
    pub headline: StyleScales,
    pub title: StyleScales,
    pub body: StyleScales,
    pub label: StyleScales,
}

impl Default for ThemeTypography {
    fn default() -> Self {
        let regular_set = FontSet {
            regular: "FiraSans-Regular.ttf".into(),
            bold: "FiraSans-Bold.ttf".into(),
            italic: "FiraSans-Italic.ttf".into(),
            bold_italic: "FiraSans-BoldItalic.ttf".into(),
        };

        let medium_set = FontSet {
            regular: "FiraSans-Medium.ttf".into(),
            bold: "FiraSans-Bold.ttf".into(),
            italic: "FiraSans-MediumItalic.ttf".into(),
            bold_italic: "FiraSans-BoldItalic.ttf".into(),
        };

        let condensed_set = FontSet {
            regular: "FiraSansCondensed-Regular.ttf".into(),
            bold: "FiraSansCondensed-Bold.ttf".into(),
            italic: "FiraSansCondensed-Italic.ttf".into(),
            bold_italic: "FiraSansCondensed-BoldItalic.ttf".into(),
        };

        Self {
            display: StyleScales {
                small: FontConfig {
                    font: regular_set.clone(),
                    weight: 400.,
                    size: 36.,
                    tracking: 0.,
                    line_height: 44.,
                },
                medium: FontConfig {
                    font: regular_set.clone(),
                    weight: 400.,
                    size: 45.,
                    tracking: 0.,
                    line_height: 52.,
                },
                large: FontConfig {
                    font: condensed_set.clone(),
                    weight: 400.,
                    size: 57.,
                    tracking: -0.25,
                    line_height: 64.,
                },
            },
            headline: StyleScales {
                small: FontConfig {
                    font: regular_set.clone(),
                    weight: 400.,
                    size: 24.,
                    tracking: 0.,
                    line_height: 32.,
                },
                medium: FontConfig {
                    font: regular_set.clone(),
                    weight: 400.,
                    size: 28.,
                    tracking: 0.,
                    line_height: 36.,
                },
                large: FontConfig {
                    font: regular_set.clone(),
                    weight: 400.,
                    size: 32.,
                    tracking: 0.,
                    line_height: 40.,
                },
            },
            title: StyleScales {
                small: FontConfig {
                    font: medium_set.clone(),
                    weight: 500.,
                    size: 14.,
                    tracking: 0.1,
                    line_height: 20.,
                },
                medium: FontConfig {
                    font: medium_set.clone(),
                    weight: 500.,
                    size: 16.,
                    tracking: 0.15,
                    line_height: 24.,
                },
                large: FontConfig {
                    font: regular_set.clone(),
                    weight: 400.,
                    size: 22.,
                    tracking: 0.,
                    line_height: 28.,
                },
            },
            body: StyleScales {
                small: FontConfig {
                    font: regular_set.clone(),
                    weight: 400.,
                    size: 12.,
                    tracking: 0.4,
                    line_height: 16.,
                },
                medium: FontConfig {
                    font: regular_set.clone(),
                    weight: 400.,
                    size: 14.,
                    tracking: 0.25,
                    line_height: 20.,
                },
                large: FontConfig {
                    font: regular_set.clone(),
                    weight: 400.,
                    size: 16.,
                    tracking: 0.5,
                    line_height: 24.,
                },
            },
            label: StyleScales {
                small: FontConfig {
                    font: medium_set.clone(),
                    weight: 500.,
                    size: 11.,
                    tracking: 0.5,
                    line_height: 16.,
                },
                medium: FontConfig {
                    font: medium_set.clone(),
                    weight: 500.,
                    size: 12.,
                    tracking: 0.5,
                    line_height: 16.,
                },
                large: FontConfig {
                    font: medium_set.clone(),
                    weight: 500.,
                    size: 14.,
                    tracking: 0.1,
                    line_height: 20.,
                },
            },
        }
    }
}
