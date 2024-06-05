use bevy::prelude::*;

use crate::ui_builder::UiBuilder;

use super::menu_item::{MenuItem, MenuItemConfig, MenuItemUpdate, UiMenuItemExt};

pub struct ToggleMenuItemPlugin;

impl Plugin for ToggleMenuItemPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, ToggleMenuItemUpdate.after(MenuItemUpdate))
            .add_systems(
                Update,
                (update_toggle_menu_item_value, update_toggle_menu_checkmark)
                    .chain()
                    .in_set(ToggleMenuItemUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct ToggleMenuItemUpdate;

fn update_toggle_menu_item_value(
    mut q_menu_items: Query<(&mut ToggleMenuItem, &MenuItem), Changed<MenuItem>>,
) {
    for (mut toggle, menu_item) in &mut q_menu_items {
        if menu_item.interacted() {
            toggle.checked = !toggle.checked;
        }
    }
}

fn update_toggle_menu_checkmark(
    mut q_menu_items: Query<(&ToggleMenuItem, &mut MenuItemConfig), Changed<ToggleMenuItem>>,
) {
    for (toggle, mut config) in &mut q_menu_items {
        if toggle.checked {
            config.leading_icon = "embedded://sickle_ui/icons/checkmark.png"
                .to_string()
                .into();
        } else {
            config.leading_icon = None;
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ToggleMenuItem {
    pub checked: bool,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ToggleMenuItemConfig {
    pub name: String,
    pub alt_code: Option<KeyCode>,
    pub shortcut: Option<Vec<KeyCode>>,
    pub initially_checked: bool,
}

impl Into<MenuItemConfig> for ToggleMenuItemConfig {
    fn into(self) -> MenuItemConfig {
        MenuItemConfig {
            name: self.name,
            alt_code: self.alt_code,
            shortcut: self.shortcut,
            ..default()
        }
    }
}

pub trait UiToggleMenuItemExt<'w, 's> {
    fn toggle_menu_item<'a>(
        &'a mut self,
        config: ToggleMenuItemConfig,
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiToggleMenuItemExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn toggle_menu_item<'a>(
        &'a mut self,
        config: ToggleMenuItemConfig,
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut item = self.menu_item(config.clone().into());

        let name = format!("Toggle Menu [{}]", config.name.clone());
        item.insert((
            Name::new(name),
            ToggleMenuItem {
                checked: config.initially_checked,
            },
            config,
        ));

        item
    }
}
