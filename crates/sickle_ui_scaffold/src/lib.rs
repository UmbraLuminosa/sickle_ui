pub mod drag_interaction;
pub mod drop_interaction;
pub mod flux_interaction;
pub mod scroll_interaction;
pub mod theme;
pub mod ui_builder;
pub mod ui_commands;
pub mod ui_style;
pub mod ui_utils;

use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use super::{
        drag_interaction::{DragState, Draggable, DraggableUpdate},
        drop_interaction::{DropPhase, DropZone, Droppable, DroppableUpdate},
        flux_interaction::{
            FluxInteraction, FluxInteractionStopwatch, FluxInteractionStopwatchLock,
            FluxInteractionUpdate, TrackedInteraction,
        },
        scroll_interaction::{ScrollAxis, Scrollable, ScrollableUpdate},
        theme::prelude::*,
        ui_builder::{UiBuilder, UiBuilderExt, UiContextRoot, UiRoot},
        ui_commands::ManagePseudoStateExt,
        ui_style::prelude::*,
        ui_utils::UiUtils,
        CardinalDirection,
    };
}

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
