use bevy::prelude::*;

mod assets;
pub mod dev_panels;
pub mod hierarchy_delay;
pub mod input_extension;
pub mod widgets;

use animated_interaction::AnimatedInteractionPlugin;
use assets::BuiltInAssetsPlugin;
use drag_interaction::DragInteractionPlugin;
use drop_interaction::DropInteractionPlugin;
use hierarchy_delay::HierarchyDelayPlugin;
use interactions::InteractionsPlugin;
use resize_interaction::ResizeHandlePlugin;
use scroll_interaction::ScrollInteractionPlugin;
use theme::ThemePlugin;
use widgets::WidgetsPlugin;

pub use sickle_math::*;
pub use sickle_ui_scaffold::*;

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
