use std::char;

use bevy::prelude::*;

#[derive(Clone, Debug, Default, Reflect)]
pub enum IconData {
    #[default]
    None,
    Image(String, Color),
    FontCodepoint(String, char, Color, f32),
    // TODO: add texture atlas config
}

impl IconData {
    pub fn with_color(&self, color: Color) -> Self {
        match self {
            IconData::None => IconData::None,
            IconData::Image(path, _) => Self::Image(path.clone(), color),
            IconData::FontCodepoint(path, codepoint, _, size) => {
                Self::FontCodepoint(path.clone(), codepoint.clone(), color, size.clone())
            }
        }
    }

    pub fn with_size(&self, size: f32) -> Self {
        match self {
            IconData::None => IconData::None,
            IconData::Image(_, _) => self.clone(),
            IconData::FontCodepoint(path, codepoint, color, _) => {
                Self::FontCodepoint(path.clone(), codepoint.clone(), color.clone(), size)
            }
        }
    }

    pub fn with(&self, color: Color, size: f32) -> Self {
        match self {
            IconData::None => IconData::None,
            IconData::Image(path, _) => Self::Image(path.clone(), color),
            IconData::FontCodepoint(path, codepoint, _, _) => {
                Self::FontCodepoint(path.clone(), codepoint.clone(), color, size)
            }
        }
    }
}

#[derive(Clone, Debug, Default, Reflect)]
pub struct CustomIconData {
    pub name: String,
    pub data: IconData,
}

#[derive(Clone, Debug, Reflect)]
pub struct Icons {
    pub checkbox_unchecked: IconData,
    pub checkbox_checked: IconData,
    pub checkmark: IconData,
    pub chevron_down: IconData,
    pub chevron_left: IconData,
    pub chevron_right: IconData,
    pub chevron_up: IconData,
    pub close: IconData,
    pub exit: IconData,
    pub popout: IconData,
    pub redo: IconData,
    pub submenu: IconData,
    pub undo: IconData,
    pub custom: Vec<CustomIconData>,
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            checkbox_unchecked: IconData::Image("".into(), Color::WHITE),
            checkbox_checked: IconData::Image("".into(), Color::WHITE),
            checkmark: IconData::FontCodepoint(
                "embedded://sickle_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E5CA}',
                Color::WHITE,
                12.,
            ),
            chevron_down: IconData::FontCodepoint(
                "embedded://sickle_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E5CF}',
                Color::WHITE,
                12.,
            ),
            chevron_left: IconData::Image("".into(), Color::WHITE),
            chevron_right: IconData::Image("".into(), Color::WHITE),
            chevron_up: IconData::Image("".into(), Color::WHITE),
            close: IconData::Image("".into(), Color::WHITE),
            exit: IconData::Image("".into(), Color::WHITE),
            popout: IconData::Image("".into(), Color::WHITE),
            redo: IconData::Image("".into(), Color::WHITE),
            submenu: IconData::Image("".into(), Color::WHITE),
            undo: IconData::Image("".into(), Color::WHITE),
            custom: Vec::new(),
        }
    }
}
