pub mod checkbox;
pub mod column;
pub mod container;
pub mod context_menu;
pub mod docking_zone;
pub mod dropdown;
pub mod floating_panel;
pub mod foldable;
pub mod icon;
pub mod label;
pub mod menu;
pub mod menu_item;
pub mod panel;
pub mod radio_group;
pub mod row;
pub mod scroll_view;
pub mod sized_zone;
pub mod slider;
pub mod submenu;
pub mod tab_container;
pub mod toggle_menu_item;

use bevy::prelude::*;

use self::{
    checkbox::CheckboxPlugin,
    context_menu::ContextMenuPlugin,
    docking_zone::DockingZonePlugin,
    dropdown::DropdownPlugin,
    floating_panel::{FloatingPanelPlugin, FloatingPanelUpdate},
    foldable::FoldablePlugin,
    menu::MenuPlugin,
    menu_item::MenuItemPlugin,
    radio_group::RadioGroupPlugin,
    scroll_view::ScrollViewPlugin,
    sized_zone::SizedZonePlugin,
    slider::SliderPlugin,
    submenu::SubmenuPlugin,
    tab_container::TabContainerPlugin,
    toggle_menu_item::ToggleMenuItemPlugin,
};

pub mod prelude {
    pub use super::{
        checkbox::{Checkbox, UiCheckboxExt},
        column::UiColumnExt,
        container::UiContainerExt,
        context_menu::{ContextMenuGenerator, GenerateContextMenu, ReflectContextMenuGenerator},
        docking_zone::UiDockingZoneExt,
        dropdown::UiDropdownExt,
        floating_panel::{FloatingPanelConfig, FloatingPanelLayout, UiFloatingPanelExt},
        foldable::UiFoldableExt,
        icon::UiIconExt,
        label::{LabelConfig, SetLabelTextExt, UiLabelExt},
        menu::{
            MenuConfig, MenuItemSeparator, MenuSeparator, UiMenuExt, UiMenuItemSeparatorExt,
            UiMenuSeparatorExt,
        },
        menu_item::{MenuItem, MenuItemConfig, MenuItemUpdate, UiMenuItemExt},
        panel::UiPanelExt,
        radio_group::{RadioGroup, UiRadioGroupExt},
        row::UiRowExt,
        scroll_view::UiScrollViewExt,
        sized_zone::{SizedZoneConfig, UiSizedZoneExt},
        slider::{SliderConfig, UiSliderExt},
        submenu::{SubmenuConfig, UiSubmenuExt},
        tab_container::UiTabContainerExt,
        toggle_menu_item::{ToggleMenuItem, ToggleMenuItemConfig, UiToggleMenuItemExt},
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
            ))
            .add_plugins((
                MenuPlugin,
                MenuItemPlugin,
                RadioGroupPlugin,
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
