use bevy::{prelude::*, ui::FocusPolicy};

use sickle_ui_scaffold::prelude::*;

use crate::{
    input_extension::ShortcutTextExt,
    widgets::layout::{
        container::UiContainerExt,
        label::{LabelConfig, UiLabelExt},
    },
};

use super::{
    context_menu::{ContextMenu, ContextMenuUpdate, UiContextMenuExt},
    menu::{Menu, MenuUpdate, UiMenuSubExt},
    shortcut::Shortcut,
    submenu::{Submenu, SubmenuUpdate, UiSubmenuSubExt},
};

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
                update_menu_item_on_shortcut_press,
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
            item.bypass_change_detection().interacted = false;
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

fn update_menu_item_on_shortcut_press(
    mut q_menu_items: Query<(&mut MenuItem, &Shortcut), Changed<Shortcut>>,
) {
    for (mut item, shortcut) in &mut q_menu_items {
        if shortcut.pressed() && !item.interacted {
            item.interacted = true;
        }
    }
}

#[derive(Debug, Default)]
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
    leading_icon: IconData,
    trailing_icon: IconData,
    alt_code: Option<KeyCode>,
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
            leading_icon: IconData::None,
            trailing_icon: IconData::None,
            alt_code: None,
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

impl MenuItem {
    pub const LEADING_ICON: &'static str = "LeadingIcon";
    pub const LABEL: &'static str = "Label";
    pub const SHORTCUT_CONTAINER: &'static str = "ShortcutContainer";
    pub const SHORTCUT: &'static str = "Shortcut";
    pub const TRAILING_ICON: &'static str = "TrailingIcon";

    pub fn interacted(&self) -> bool {
        self.interacted
    }

    pub fn alt_code(&self) -> Option<KeyCode> {
        self.alt_code
    }

    pub fn leading(&self) -> Entity {
        self.leading
    }

    pub fn label(&self) -> Entity {
        self.label
    }

    pub fn shortcut_container(&self) -> Entity {
        self.shortcut_container
    }

    pub fn shortcut(&self) -> Entity {
        self.shortcut
    }

    pub fn trailing(&self) -> Entity {
        self.trailing
    }

    pub fn leading_icon(&self) -> IconData {
        self.leading_icon.clone()
    }

    pub fn trailing_icon(&self) -> IconData {
        self.trailing_icon.clone()
    }

    pub fn theme() -> Theme<MenuItem> {
        let base_theme = PseudoTheme::deferred_context(None, MenuItem::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(
        style_builder: &mut StyleBuilder,
        menu_item: &MenuItem,
        theme_data: &ThemeData,
    ) {
        let leading_icon = menu_item.leading_icon.clone();
        let trailing_icon = menu_item.trailing_icon.clone();

        MenuItem::menu_item_style(style_builder, theme_data, leading_icon, trailing_icon);
    }

    pub(crate) fn menu_item_style(
        style_builder: &mut StyleBuilder,
        theme_data: &ThemeData,
        leading_icon: IconData,
        trailing_icon: IconData,
    ) {
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
                idle: colors.container(Container::SurfaceMid),
                hover: colors.container(Container::SurfaceHighest).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(MenuItem::LEADING_ICON)
            .aspect_ratio(1.)
            .size(Val::Px(theme_spacing.icons.small))
            .icon(leading_icon.with(colors.on(On::SurfaceVariant), theme_spacing.icons.small));

        style_builder
            .switch_target(MenuItem::LABEL)
            .margin(UiRect::horizontal(Val::Px(theme_spacing.gaps.small)))
            .sized_font(font.clone())
            .font_color(colors.on(On::Surface));

        style_builder
            .switch_target(MenuItem::SHORTCUT_CONTAINER)
            .justify_content(JustifyContent::End)
            .flex_wrap(FlexWrap::NoWrap)
            .flex_grow(2.)
            .margin(UiRect::left(Val::Px(theme_spacing.areas.large)));

        style_builder
            .switch_target(MenuItem::SHORTCUT)
            .sized_font(font)
            .font_color(colors.on(On::SurfaceVariant));

        style_builder
            .switch_target(MenuItem::TRAILING_ICON)
            .aspect_ratio(1.)
            .margin(UiRect::left(Val::Px(theme_spacing.gaps.small)))
            .size(Val::Px(theme_spacing.icons.small))
            .icon(trailing_icon.with(colors.on(On::SurfaceVariant), theme_spacing.icons.small));
    }

    fn button(name: String) -> impl Bundle {
        (
            Name::new(name),
            ButtonBundle {
                style: Style {
                    overflow: Overflow::visible(),
                    ..default()
                },
                focus_policy: bevy::ui::FocusPolicy::Pass,
                ..default()
            },
            TrackedInteraction::default(),
            LockedStyleAttributes::from_vec(vec![
                LockableStyleAttribute::FocusPolicy,
                LockableStyleAttribute::Overflow,
            ]),
        )
    }

    fn shortcut_container_bundle() -> impl Bundle {
        (Name::new("Shortcut"), NodeBundle::default())
    }

    fn leading_icon_bundle() -> impl Bundle {
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

    fn trailing_icon_bundle() -> impl Bundle {
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

    pub(crate) fn scaffold(
        builder: &mut UiBuilder<Entity>,
        config: MenuItemConfig,
    ) -> (Entity, MenuItem) {
        let mut menu_item = MenuItem {
            leading_icon: config.leading_icon,
            trailing_icon: config.trailing_icon,
            alt_code: config.alt_code,
            ..default()
        };

        let name = format!("Menu Item [{}]", config.name.clone());
        let shortcut_text: String = match &config.shortcut {
            Some(vec) => vec.shortcut_text().into(),
            None => "".into(),
        };

        let mut item = builder.container(MenuItem::button(name), |container| {
            menu_item.leading = container.spawn(MenuItem::leading_icon_bundle()).id();
            menu_item.label = container
                .label(LabelConfig {
                    label: config.name,
                    ..default()
                })
                .id();
            menu_item.shortcut_container = container
                .container(
                    MenuItem::shortcut_container_bundle(),
                    |shortcut_container| {
                        menu_item.shortcut = shortcut_container
                            .label(LabelConfig {
                                label: shortcut_text,
                                ..default()
                            })
                            .id();
                    },
                )
                .id();

            menu_item.trailing = container.spawn(MenuItem::trailing_icon_bundle()).id();
        });

        if let Some(shortcut) = config.shortcut {
            item.insert(Shortcut::new(shortcut));
        }

        (item.id(), menu_item)
    }
}

pub trait UiMenuItemExt {
    fn menu_item(&mut self, config: MenuItemConfig) -> UiBuilder<Entity>;
}

impl UiMenuItemExt for UiBuilder<'_, Entity> {
    fn menu_item(&mut self, config: MenuItemConfig) -> UiBuilder<Entity> {
        let (id, menu_item) = MenuItem::scaffold(self, config);

        self.commands().ui_builder(id).insert(menu_item);
        self.commands().ui_builder(id)
    }
}

impl UiMenuItemExt for UiBuilder<'_, Menu> {
    fn menu_item(&mut self, config: MenuItemConfig) -> UiBuilder<Entity> {
        let container_id = self.container();
        let id = self
            .commands()
            .ui_builder(container_id)
            .menu_item(config)
            .id();

        self.commands().ui_builder(id)
    }
}

impl UiMenuItemExt for UiBuilder<'_, Submenu> {
    fn menu_item(&mut self, config: MenuItemConfig) -> UiBuilder<Entity> {
        let container_id = self.container();
        let id = self
            .commands()
            .ui_builder(container_id)
            .menu_item(config)
            .id();

        self.commands().ui_builder(id)
    }
}

impl UiMenuItemExt for UiBuilder<'_, ContextMenu> {
    fn menu_item(&mut self, config: MenuItemConfig) -> UiBuilder<Entity> {
        let container_id = self.container();
        let id = self
            .commands()
            .ui_builder(container_id)
            .menu_item(config)
            .id();

        self.commands().ui_builder(id)
    }
}
