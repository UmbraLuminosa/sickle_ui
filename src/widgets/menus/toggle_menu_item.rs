use bevy::prelude::*;

use sickle_ui_scaffold::prelude::*;

use super::{
    context_menu::{ContextMenu, UiContextMenuExt},
    menu::{Menu, UiMenuSubExt},
    menu_item::{MenuItem, MenuItemConfig, MenuItemUpdate},
    shortcut::Shortcut,
    submenu::{Submenu, UiSubmenuSubExt},
};

pub struct ToggleMenuItemPlugin;

impl Plugin for ToggleMenuItemPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, ToggleMenuItemUpdate.after(MenuItemUpdate))
            .add_plugins(ComponentThemePlugin::<ToggleMenuItem>::default())
            .add_systems(
                Update,
                (
                    update_toggle_menu_item_value,
                    update_toggle_menu_item_on_shortcut_press,
                    update_toggle_menu_checkmark,
                )
                    .chain()
                    .in_set(ToggleMenuItemUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct ToggleMenuItemUpdate;

fn update_toggle_menu_item_value(
    mut q_menu_items: Query<(&mut ToggleMenuItem, &FluxInteraction), Changed<FluxInteraction>>,
) {
    for (mut toggle, interaction) in &mut q_menu_items {
        if interaction.is_pressed() {
            toggle.checked = !toggle.checked;
        }
    }
}

fn update_toggle_menu_item_on_shortcut_press(
    mut q_menu_items: Query<(&mut ToggleMenuItem, &Shortcut), Changed<Shortcut>>,
) {
    for (mut toggle, shortcut) in &mut q_menu_items {
        if shortcut.pressed() {
            toggle.checked = !toggle.checked;
        }
    }
}

fn update_toggle_menu_checkmark(
    q_menu_items: Query<(Entity, &ToggleMenuItem), Changed<ToggleMenuItem>>,
    mut commands: Commands,
) {
    for (entity, toggle) in &q_menu_items {
        if toggle.checked {
            commands
                .entity(entity)
                .add_pseudo_state(PseudoState::Checked);
        } else {
            commands
                .entity(entity)
                .remove_pseudo_state(PseudoState::Checked);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ToggleMenuItemConfig {
    pub name: String,
    pub trailing_icon: IconData,
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
            trailing_icon: self.trailing_icon,
            ..default()
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct ToggleMenuItem {
    pub checked: bool,
    leading: Entity,
    label: Entity,
    shortcut_container: Entity,
    shortcut: Entity,
    trailing: Entity,
    trailing_icon: IconData,
    alt_code: Option<KeyCode>,
}

impl Default for ToggleMenuItem {
    fn default() -> Self {
        Self {
            checked: Default::default(),
            leading: Entity::PLACEHOLDER,
            label: Entity::PLACEHOLDER,
            shortcut_container: Entity::PLACEHOLDER,
            shortcut: Entity::PLACEHOLDER,
            trailing: Entity::PLACEHOLDER,
            trailing_icon: Default::default(),
            alt_code: Default::default(),
        }
    }
}

impl Into<ToggleMenuItem> for MenuItem {
    fn into(self) -> ToggleMenuItem {
        ToggleMenuItem {
            checked: false,
            label: self.label(),
            leading: self.leading(),
            shortcut_container: self.shortcut_container(),
            shortcut: self.shortcut(),
            trailing: self.trailing(),
            trailing_icon: self.trailing_icon(),
            alt_code: self.alt_code(),
        }
    }
}

impl DefaultTheme for ToggleMenuItem {
    fn default_theme() -> Option<Theme<ToggleMenuItem>> {
        ToggleMenuItem::theme().into()
    }
}

impl UiContext for ToggleMenuItem {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            MenuItem::LEADING_ICON => Ok(self.leading),
            MenuItem::LABEL => Ok(self.label),
            MenuItem::SHORTCUT_CONTAINER => Ok(self.shortcut_container),
            MenuItem::SHORTCUT => Ok(self.shortcut),
            MenuItem::TRAILING_ICON => Ok(self.trailing),
            _ => Err(format!(
                "{} doesn't exists for MenuItem. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![
            MenuItem::LEADING_ICON,
            MenuItem::LABEL,
            MenuItem::SHORTCUT_CONTAINER,
            MenuItem::SHORTCUT,
            MenuItem::TRAILING_ICON,
        ]
    }
}

impl ToggleMenuItem {
    pub fn theme() -> Theme<ToggleMenuItem> {
        let base_theme = PseudoTheme::deferred_context(None, ToggleMenuItem::primary_style);
        let checked_theme =
            PseudoTheme::deferred(vec![PseudoState::Checked], ToggleMenuItem::checked_style);
        Theme::new(vec![base_theme, checked_theme])
    }

    fn primary_style(
        style_builder: &mut StyleBuilder,
        menu_item: &ToggleMenuItem,
        theme_data: &ThemeData,
    ) {
        let leading_icon = theme_data.icons.checkmark.clone();
        let trailing_icon = menu_item.trailing_icon.clone();

        MenuItem::menu_item_style(style_builder, theme_data, leading_icon, trailing_icon);

        style_builder
            .switch_target(MenuItem::LEADING_ICON)
            .visibility(Visibility::Hidden);
    }

    fn checked_style(style_builder: &mut StyleBuilder, _: &ThemeData) {
        style_builder
            .switch_target(MenuItem::LEADING_ICON)
            .visibility(Visibility::Inherited);
    }
}

pub trait UiToggleMenuItemExt {
    fn toggle_menu_item(&mut self, config: ToggleMenuItemConfig) -> UiBuilder<Entity>;
}

impl UiToggleMenuItemExt for UiBuilder<'_, Entity> {
    fn toggle_menu_item(&mut self, config: ToggleMenuItemConfig) -> UiBuilder<Entity> {
        let checked = config.initially_checked;
        let (id, menu_item) = MenuItem::scaffold(self, config.into());
        let toggle_item = ToggleMenuItem {
            checked,
            ..menu_item.into()
        };

        self.commands().ui_builder(id).insert(toggle_item);
        self.commands().ui_builder(id)
    }
}

impl UiToggleMenuItemExt for UiBuilder<'_, Menu> {
    fn toggle_menu_item(&mut self, config: ToggleMenuItemConfig) -> UiBuilder<Entity> {
        let container_id = self.container();
        let id = self
            .commands()
            .ui_builder(container_id)
            .toggle_menu_item(config)
            .id();

        self.commands().ui_builder(id)
    }
}

impl UiToggleMenuItemExt for UiBuilder<'_, Submenu> {
    fn toggle_menu_item(&mut self, config: ToggleMenuItemConfig) -> UiBuilder<Entity> {
        let container_id = self.container();
        let id = self
            .commands()
            .ui_builder(container_id)
            .toggle_menu_item(config)
            .id();

        self.commands().ui_builder(id)
    }
}

impl UiToggleMenuItemExt for UiBuilder<'_, ContextMenu> {
    fn toggle_menu_item(&mut self, config: ToggleMenuItemConfig) -> UiBuilder<Entity> {
        let container_id = self.container();
        let id = self
            .commands()
            .ui_builder(container_id)
            .toggle_menu_item(config)
            .id();

        self.commands().ui_builder(id)
    }
}
