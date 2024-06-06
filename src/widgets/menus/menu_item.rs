use bevy::{prelude::*, ui::FocusPolicy};
use sickle_ui_scaffold::{
    theme::{
        icons::IconData,
        theme_colors::{Accent, Container, On},
        theme_data::ThemeData,
        typography::{FontScale, FontStyle, FontType},
        ComponentThemePlugin, DefaultTheme, PseudoTheme, Theme, UiContext,
    },
    ui_commands::RefreshThemeExt,
    ui_style::{AnimatedVals, LockableStyleAttribute, LockedStyleAttributes, StyleBuilder},
};

use crate::{
    input_extension::{ShortcutTextExt, SymmetricKeysExt},
    ui_builder::*,
    widgets::prelude::{LabelConfig, SetLabelTextExt, UiContainerExt, UiLabelExt},
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use super::{
    context_menu::ContextMenuUpdate,
    menu::{Menu, MenuUpdate, UiMenuSubExt},
    submenu::SubmenuUpdate,
};

const LEADING_ICON: &'static str = "LeadingIcon";
const LABEL: &'static str = "Label";
const SHORTCUT_CONTAINER: &'static str = "ShortcutContainer";
const SHORTCUT: &'static str = "Shortcut";
const TRAILING_ICON: &'static str = "TrailingIcon";

pub struct MenuItemPlugin;

impl Plugin for MenuItemPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            MenuItemUpdate
                .after(FluxInteractionUpdate)
                .before(MenuUpdate)
                .before(SubmenuUpdate)
                .before(ContextMenuUpdate),
        )
        .add_plugins(ComponentThemePlugin::<MenuItem>::default())
        .add_systems(
            Update,
            (
                update_menu_item_on_change,
                update_menu_item_on_pressed,
                update_menu_item_on_key_press,
                update_menu_item_on_config_change,
            )
                .chain()
                .in_set(MenuItemUpdate),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct MenuItemUpdate;

fn update_menu_item_on_change(mut q_menu_items: Query<&mut MenuItem, Changed<MenuItem>>) {
    for mut item in &mut q_menu_items {
        if item.interacted {
            item.interacted = false;
        }
    }
}

fn update_menu_item_on_pressed(
    mut q_menu_items: Query<(&mut MenuItem, &FluxInteraction), Changed<FluxInteraction>>,
) {
    for (mut item, interaction) in &mut q_menu_items {
        if *interaction == FluxInteraction::Released {
            item.interacted = true;
        }
    }
}

fn update_menu_item_on_key_press(
    mut q_menu_items: Query<(&mut MenuItem, &MenuItemConfig)>,
    r_keys: Res<ButtonInput<KeyCode>>,
) {
    if !r_keys.is_changed() {
        return;
    }

    for (mut item, config) in &mut q_menu_items {
        if let Some(shortcut) = &config.shortcut {
            if shortcut.len() == 0 {
                continue;
            }

            let main_key = shortcut.last().unwrap().clone();
            if r_keys.just_pressed(main_key) {
                if shortcut.len() > 1 {
                    if shortcut
                        .iter()
                        .take(shortcut.len() - 1)
                        .map(|c| c.clone())
                        .all(|keycode| r_keys.symmetry_pressed(keycode))
                    {
                        item.interacted = true;
                    }
                } else {
                    item.interacted = true;
                }
            }
        }
    }
}

fn update_menu_item_on_config_change(
    q_menu_items: Query<(Entity, &MenuItem, &MenuItemConfig), Changed<MenuItemConfig>>,
    mut commands: Commands,
) {
    for (entity, menu_item, config) in &q_menu_items {
        let name = config.name.clone();
        let shortcut_text: Option<String> = match &config.shortcut {
            Some(vec) => vec.shortcut_text().into(),
            None => None,
        };

        commands.entity(menu_item.label).set_label_text(name);

        if let Some(shortcut_text) = shortcut_text {
            commands
                .entity(menu_item.shortcut)
                .set_label_text(shortcut_text);
        } else {
            commands.entity(menu_item.shortcut).set_label_text("");
        }

        commands.entity(entity).refresh_theme::<MenuItem>();
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MenuItemConfig {
    pub name: String,
    pub leading_icon: IconData,
    pub trailing_icon: IconData,
    pub alt_code: Option<KeyCode>,
    pub shortcut: Option<Vec<KeyCode>>,
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct MenuItem {
    interacted: bool,
    leading: Entity,
    label: Entity,
    shortcut_container: Entity,
    shortcut: Entity,
    trailing: Entity,
}

impl Default for MenuItem {
    fn default() -> Self {
        Self {
            interacted: Default::default(),
            leading: Entity::PLACEHOLDER,
            label: Entity::PLACEHOLDER,
            shortcut_container: Entity::PLACEHOLDER,
            shortcut: Entity::PLACEHOLDER,
            trailing: Entity::PLACEHOLDER,
        }
    }
}

impl DefaultTheme for MenuItem {
    fn default_theme() -> Option<Theme<MenuItem>> {
        MenuItem::theme().into()
    }
}

impl UiContext for MenuItem {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            LEADING_ICON => Ok(self.leading),
            LABEL => Ok(self.label),
            SHORTCUT_CONTAINER => Ok(self.shortcut_container),
            SHORTCUT => Ok(self.shortcut),
            TRAILING_ICON => Ok(self.trailing),
            _ => Err(format!(
                "{} doesn't exists for MenuItem. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![
            LEADING_ICON,
            LABEL,
            SHORTCUT_CONTAINER,
            SHORTCUT,
            TRAILING_ICON,
        ]
    }
}

impl MenuItem {
    pub fn theme() -> Theme<MenuItem> {
        let base_theme = PseudoTheme::deferred_world(None, MenuItem::primary_style);
        Theme::<MenuItem>::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, entity: Entity, world: &mut World) {
        let Some(config) = world.get::<MenuItemConfig>(entity) else {
            return;
        };

        let leading_icon = config.leading_icon.clone();
        let trailing_icon = config.trailing_icon.clone();

        let theme_data = world.resource::<ThemeData>().clone();
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);

        style_builder
            .justify_content(JustifyContent::End)
            .align_items(AlignItems::Center)
            .height(Val::Px(theme_spacing.areas.small))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.extra_small)))
            .margin(UiRect::vertical(Val::Px(theme_spacing.gaps.tiny)))
            .animated()
            .background_color(AnimatedVals {
                idle: colors.container(Container::SurfaceHigh),
                hover: colors.accent(Accent::OutlineVariant).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(LEADING_ICON)
            .aspect_ratio(1.)
            .size(Val::Px(theme_spacing.icons.small))
            .icon(leading_icon.with(colors.on(On::Surface), theme_spacing.icons.small));

        style_builder
            .switch_target(LABEL)
            .margin(UiRect::horizontal(Val::Px(theme_spacing.gaps.medium)))
            .sized_font(font.clone())
            .font_color(colors.on(On::Surface));

        style_builder
            .switch_target(SHORTCUT_CONTAINER)
            .justify_content(JustifyContent::End)
            .flex_wrap(FlexWrap::NoWrap)
            .flex_grow(2.)
            .margin(UiRect::left(Val::Px(theme_spacing.areas.large)));

        style_builder
            .switch_target(SHORTCUT)
            .sized_font(font)
            .font_color(colors.on(On::Surface));

        style_builder
            .switch_target(TRAILING_ICON)
            .aspect_ratio(1.)
            .margin(UiRect::left(Val::Px(theme_spacing.gaps.small)))
            .size(Val::Px(theme_spacing.icons.small))
            .icon(trailing_icon.with(colors.on(On::Surface), theme_spacing.icons.small));
    }

    pub fn interacted(&self) -> bool {
        self.interacted
    }

    fn button(name: String) -> impl Bundle {
        (
            Name::new(name),
            ButtonBundle {
                focus_policy: bevy::ui::FocusPolicy::Pass,
                ..default()
            },
            TrackedInteraction::default(),
            LockedStyleAttributes::lock(LockableStyleAttribute::FocusPolicy),
        )
    }

    fn shortcut() -> impl Bundle {
        (Name::new("Shortcut"), NodeBundle::default())
    }

    fn leading_icon() -> impl Bundle {
        (
            Name::new("Leading Icon"),
            ImageBundle {
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            BorderColor::default(),
            LockedStyleAttributes::lock(LockableStyleAttribute::FocusPolicy),
        )
    }

    fn trailing_icon() -> impl Bundle {
        (
            Name::new("Trailing Icon"),
            ImageBundle {
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            BorderColor::default(),
            LockedStyleAttributes::lock(LockableStyleAttribute::FocusPolicy),
        )
    }
}

pub trait UiMenuItemExt<'w, 's> {
    fn menu_item<'a>(&'a mut self, config: MenuItemConfig) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiMenuItemExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn menu_item<'a>(&'a mut self, config: MenuItemConfig) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut menu_item = MenuItem::default();
        let name = format!("Menu Item [{}]", config.name.clone());

        let mut item = self.container((MenuItem::button(name), config), |container| {
            menu_item.leading = container.spawn(MenuItem::leading_icon()).id();
            menu_item.label = container.label(LabelConfig::default()).id();
            menu_item.shortcut_container = container
                .container(MenuItem::shortcut(), |shortcut_container| {
                    menu_item.shortcut = shortcut_container.label(LabelConfig::default()).id();
                })
                .id();

            menu_item.trailing = container.spawn(MenuItem::trailing_icon()).id();
        });

        item.insert(menu_item);

        item
    }
}

impl<'w, 's> UiMenuItemExt<'w, 's> for UiBuilder<'w, 's, '_, Menu> {
    fn menu_item<'a>(&'a mut self, config: MenuItemConfig) -> UiBuilder<'w, 's, 'a, Entity> {
        let container_id = self.container();
        let id = self
            .commands()
            .ui_builder(container_id)
            .menu_item(config)
            .id();

        self.commands().ui_builder(id)
    }
}

// TODO: Also add to context menu
// impl<'w, 's> UiMenuItemExt<'w, 's> for UiBuilder<'w, 's, '_, ContextMenu> {
//     fn menu_item<'a>(&'a mut self, config: MenuItemConfig) -> UiBuilder<'w, 's, 'a, Entity> {
//         let container_id = self.container();
//         let id = self
//             .commands()
//             .ui_builder(container_id)
//             .menu_item(config)
//             .id();

//         self.commands().ui_builder(id)
//     }
// }
