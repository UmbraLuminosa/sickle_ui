pub mod checkbox;
pub mod column;
pub mod container;
pub mod docking_zone;
pub mod dropdown;
pub mod floating_panel;
pub mod foldable;
pub mod icon;
pub mod label;
pub mod menus;
pub mod panel;
pub mod radio_group;
pub mod resize_handles;
pub mod row;
pub mod scroll_view;
pub mod sized_zone;
pub mod slider;
pub mod tab_container;

use bevy::prelude::*;
use menus::menu_separators::MenuSeparatorPlugin;

use self::{
    checkbox::CheckboxPlugin,
    docking_zone::DockingZonePlugin,
    dropdown::DropdownPlugin,
    floating_panel::{FloatingPanelPlugin, FloatingPanelUpdate},
    foldable::FoldablePlugin,
    menus::context_menu::ContextMenuPlugin,
    menus::menu::MenuPlugin,
    menus::menu_bar::MenuBarPlugin,
    menus::menu_item::MenuItemPlugin,
    menus::shortcut::ShortcutPlugin,
    menus::submenu::SubmenuPlugin,
    menus::toggle_menu_item::ToggleMenuItemPlugin,
    radio_group::RadioGroupPlugin,
    resize_handles::ResizeHandlePlugin,
    scroll_view::ScrollViewPlugin,
    sized_zone::SizedZonePlugin,
    slider::SliderPlugin,
    tab_container::TabContainerPlugin,
};

// TODO: Re-organize prelude
pub mod prelude {
    pub use super::{
        checkbox::{Checkbox, UiCheckboxExt},
        column::UiColumnExt,
        container::UiContainerExt,
        docking_zone::UiDockingZoneExt,
        dropdown::*,
        floating_panel::{FloatingPanelConfig, FloatingPanelLayout, UiFloatingPanelExt},
        foldable::UiFoldableExt,
        icon::UiIconExt,
        label::{LabelConfig, SetLabelTextExt, UiLabelExt},
        menus::context_menu::{
            ContextMenuGenerator, GenerateContextMenu, ReflectContextMenuGenerator,
        },
        menus::menu::*,
        menus::menu_bar::*,
        menus::menu_item::*,
        menus::menu_separators::*,
        menus::shortcut::*,
        menus::submenu::{SubmenuConfig, UiSubmenuExt},
        menus::toggle_menu_item::{ToggleMenuItem, ToggleMenuItemConfig, UiToggleMenuItemExt},
        panel::UiPanelExt,
        radio_group::{RadioGroup, UiRadioGroupExt},
        resize_handles::*,
        row::UiRowExt,
        scroll_view::UiScrollViewExt,
        sized_zone::{SizedZoneConfig, UiSizedZoneExt},
        slider::{SliderConfig, UiSliderExt},
        tab_container::*,
        WidgetLibraryUpdate,
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
