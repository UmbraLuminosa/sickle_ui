pub mod animated_interaction;
pub mod drag_interaction;
pub mod drop_interaction;
pub mod flux_interaction;
pub mod interactions;
pub mod scroll_interaction;
pub mod theme;
pub mod ui_builder;
pub mod ui_commands;
pub mod ui_style;
pub mod ui_utils;

use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

pub use self::flux_interaction::*;
pub use self::ui_utils::UiUtils;

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Reflect, Serialize, Deserialize,
)]
pub enum CardinalDirection {
    #[default]
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}
