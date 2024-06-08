pub mod inputs;
pub mod layout;
pub mod menus;

use bevy::prelude::*;

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
    menus::menu_separators::MenuSeparatorPlugin,
    menus::shortcut::ShortcutPlugin,
    menus::submenu::SubmenuPlugin,
    menus::toggle_menu_item::ToggleMenuItemPlugin,
};

pub mod prelude {
    pub use super::{
        inputs::checkbox::{Checkbox, UiCheckboxExt},
        inputs::dropdown::{Dropdown, UiDropdownExt},
        inputs::radio_group::{RadioGroup, UiRadioGroupExt},
        inputs::slider::{Slider, UiSliderExt},
        layout::column::UiColumnExt,
        layout::container::UiContainerExt,
        layout::docking_zone::UiDockingZoneExt,
        layout::floating_panel::{
            FloatingPanelConfig, FloatingPanelLayout, FloatingPanelUpdate, UiFloatingPanelExt,
        },
        layout::foldable::{Foldable, FoldableUpdate, UiFoldableExt},
        layout::icon::UiIconExt,
        layout::label::{LabelConfig, UiLabelExt},
        layout::panel::UiPanelExt,
        layout::resize_handles::{ResizeHandle, UiResizeHandlesExt},
        layout::row::UiRowExt,
        layout::scroll_view::{ScrollViewLayoutUpdate, UiScrollViewExt},
        layout::sized_zone::{SizedZoneConfig, SizedZonePreUpdate, UiSizedZoneExt},
        layout::tab_container::{TabContainerUpdate, UiTabContainerExt, UiTabContainerSubExt},
        menus::context_menu::{
            ContextMenuGenerator, ContextMenuUpdate, ReflectContextMenuGenerator, UiContextMenuExt,
        },
        menus::extra_menu::{ExtraMenu, UiExtraMenuExt},
        menus::menu::{MenuConfig, MenuUpdate, UiMenuExt, UiMenuSubExt},
        menus::menu_bar::{UiMenuBarExt, UiMenuBarSubExt},
        menus::menu_item::{MenuItem, MenuItemConfig, MenuItemUpdate, UiMenuItemExt},
        menus::menu_separators::{UiMenuItemSeparatorExt, UiMenuSeparatorExt},
        menus::shortcut::{Shortcut, ShortcutPreUpdate},
        menus::submenu::{SubmenuConfig, SubmenuUpdate, UiSubmenuExt, UiSubmenuSubExt},
        menus::toggle_menu_item::{
            ToggleMenuItemConfig, ToggleMenuItemUpdate, UiToggleMenuItemExt,
        },
        WidgetLibraryUpdate,
    };

    // Used with scroll views, floating panels, etc. often
    pub use sickle_ui_scaffold::scroll_interaction::ScrollAxis;
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
