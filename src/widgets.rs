pub mod inputs;
pub mod layout;
pub mod menus;

use bevy::prelude::*;
use menus::menu_separators::MenuSeparatorPlugin;

use self::{
    inputs::checkbox::CheckboxPlugin,
    inputs::dropdown::DropdownPlugin,
    inputs::radio_group::RadioGroupPlugin,
    inputs::slider::SliderPlugin,
    layout::docking_zone::DockingZonePlugin,
    layout::floating_panel::{FloatingPanelPlugin, FloatingPanelUpdate},
    layout::foldable::FoldablePlugin,
    layout::resize_handles::ResizeHandlePlugin,
    layout::scroll_view::ScrollViewPlugin,
    layout::sized_zone::SizedZonePlugin,
    layout::tab_container::TabContainerPlugin,
    menus::context_menu::ContextMenuPlugin,
    menus::menu::MenuPlugin,
    menus::menu_bar::MenuBarPlugin,
    menus::menu_item::MenuItemPlugin,
    menus::shortcut::ShortcutPlugin,
    menus::submenu::SubmenuPlugin,
    menus::toggle_menu_item::ToggleMenuItemPlugin,
};

// TODO: Re-organize prelude
pub mod prelude {
    pub use super::{
        inputs::checkbox::*, inputs::dropdown::*, inputs::radio_group::*, inputs::slider::*,
        layout::column::*, layout::container::*, layout::docking_zone::*,
        layout::floating_panel::*, layout::foldable::*, layout::icon::*, layout::label::*,
        layout::panel::*, layout::resize_handles::*, layout::row::*, layout::scroll_view::*,
        layout::sized_zone::*, layout::tab_container::*, menus::context_menu::*, menus::menu::*,
        menus::menu_bar::*, menus::menu_item::*, menus::menu_separators::*, menus::shortcut::*,
        menus::submenu::*, menus::toggle_menu_item::*, WidgetLibraryUpdate,
    };
}

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, WidgetLibraryUpdate.after(FloatingPanelUpdate))
            .add_plugins((
                CheckboxPlugin,
                ContextMenuPlugin,
                SizedZonePlugin,
                DockingZonePlugin,
                DropdownPlugin,
                FloatingPanelPlugin,
                FoldablePlugin,
                MenuPlugin,
            ))
            .add_plugins((
                MenuBarPlugin,
                MenuItemPlugin,
                MenuSeparatorPlugin,
                RadioGroupPlugin,
                ResizeHandlePlugin,
                ShortcutPlugin,
                SliderPlugin,
                ScrollViewPlugin,
                SubmenuPlugin,
                TabContainerPlugin,
                ToggleMenuItemPlugin,
            ));
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct WidgetLibraryUpdate;
