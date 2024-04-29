use bevy::prelude::*;

pub mod animated_interaction;
mod assets;
pub mod dev_panels;
pub mod drag_interaction;
pub mod drop_interaction;
pub mod flux_interaction;
pub mod hierarchy_delay;
pub mod input_extension;
pub mod interactions;
pub mod resize_interaction;
pub mod scroll_interaction;
pub mod theme;
pub mod ui_builder;
pub mod ui_commands;
pub mod ui_style;
pub mod widgets;

use assets::BuiltInAssetsPlugin;
use drag_interaction::DragInteractionPlugin;
use drop_interaction::DropInteractionPlugin;
pub use flux_interaction::*;
use hierarchy_delay::HierarchyDelayPlugin;
use interactions::InteractionsPlugin;
use resize_interaction::ResizeHandlePlugin;
use scroll_interaction::ScrollInteractionPlugin;
use theme::ThemePlugin;
use widgets::WidgetsPlugin;

use self::animated_interaction::AnimatedInteractionPlugin;

pub use sickle_math::*;

/// Core plugin.
///
/// Must be added after [`DefaultPlugins`].
pub struct SickleUiPlugin;

impl Plugin for SickleUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BuiltInAssetsPlugin,
            AnimatedInteractionPlugin,
            DragInteractionPlugin,
            DropInteractionPlugin,
            HierarchyDelayPlugin,
            FluxInteractionPlugin,
            InteractionsPlugin,
            ResizeHandlePlugin,
            ScrollInteractionPlugin,
            WidgetsPlugin,
            ThemePlugin,
        ));
    }
}
