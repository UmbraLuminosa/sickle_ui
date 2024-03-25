use bevy::prelude::*;
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    input_extension::{ShortcutTextExt, SymmetricKeysExt},
    interactions::InteractiveBackground,
    ui_builder::*,
    ui_style::{SetBackgroundColorExt, SetImageExt, UiStyleExt},
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use super::{
    context_menu::ContextMenuUpdate,
    menu::MenuUpdate,
    prelude::{LabelConfig, SetLabelTextExt, UiContainerExt, UiLabelExt},
    submenu::SubmenuUpdate,
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

fn update_menu_item_on_change(mut q_menu_items: Query<&mut MenuItem, Changed<MenuItem>>) {
    for mut item in &mut q_menu_items {
        if item.interacted {
            item.interacted = false;
        }
    }
}

fn update_menu_item_on_config_change(
    q_menu_items: Query<(&MenuItem, &MenuItemConfig), Changed<MenuItemConfig>>,
    mut commands: Commands,
) {
    for (menu_item, config) in &q_menu_items {
        let name = config.name.clone();
        let shortcut_text: Option<String> = match &config.shortcut {
            Some(vec) => vec.shortcut_text().into(),
            None => None,
        };
        let leading = config.leading_icon.clone();
        let trailing = config.trailing_icon.clone();

        if let Some(leading) = leading {
            commands
                .entity(menu_item.leading)
                .try_insert(UiImage::default());
            commands
                .style(menu_item.leading)
                .image(leading)
                .background_color(Color::WHITE);
        } else {
            commands.entity(menu_item.leading).remove::<UiImage>();
            commands
                .style(menu_item.leading)
                .background_color(Color::NONE);
        }

        commands.entity(menu_item.label).set_label_text(name);

        if let Some(shortcut_text) = shortcut_text {
            commands
                .entity(menu_item.shortcut)
                .set_label_text(shortcut_text);
        } else {
            commands.entity(menu_item.shortcut).set_label_text("");
        }

        if let Some(trailing) = trailing {
            commands
                .entity(menu_item.trailing)
                .try_insert(UiImage::default());
            commands
                .style(menu_item.trailing)
                .image(trailing)
                .background_color(Color::WHITE);
        } else {
            commands.entity(menu_item.trailing).remove::<UiImage>();
            commands
                .style(menu_item.trailing)
                .background_color(Color::NONE);
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct MenuItem {
    interacted: bool,
    leading: Entity,
    label: Entity,
    shortcut: Entity,
    trailing: Entity,
}

impl Default for MenuItem {
    fn default() -> Self {
        Self {
            interacted: Default::default(),
            leading: Entity::PLACEHOLDER,
            label: Entity::PLACEHOLDER,
            shortcut: Entity::PLACEHOLDER,
            trailing: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MenuItemConfig {
    pub name: String,
    pub leading_icon: Option<String>,
    pub trailing_icon: Option<String>,
    pub alt_code: Option<KeyCode>,
    pub shortcut: Option<Vec<KeyCode>>,
    pub is_submenu: bool,
}

impl MenuItem {
    pub fn interacted(&self) -> bool {
        self.interacted
    }

    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn button() -> impl Bundle {
        (
            ButtonBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(5.)),
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::NONE.into(),
                focus_policy: bevy::ui::FocusPolicy::Pass,
                ..default()
            },
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Color::rgba(0.9, 0.8, 0.7, 0.5).into(),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: MenuItem::base_tween(),
                ..default()
            },
        )
    }

    fn shortcut() -> impl Bundle {
        (
            Name::new("Menu Item Shortcut"),
            NodeBundle {
                style: Style {
                    margin: UiRect::left(Val::Px(50.)),
                    justify_content: JustifyContent::End,
                    flex_wrap: FlexWrap::NoWrap,
                    flex_grow: 2.,
                    ..default()
                },
                ..default()
            },
        )
    }

    fn leading_icon() -> impl Bundle {
        (
            Name::new("Menu Item Leading Icon"),
            ImageBundle {
                style: Style {
                    width: Val::Px(12.),
                    aspect_ratio: (1.).into(),
                    ..default()
                },
                ..default()
            },
        )
    }

    fn trailing_icon() -> impl Bundle {
        (
            Name::new("Menu Item Trailing Icon"),
            ImageBundle {
                style: Style {
                    width: Val::Px(12.),
                    aspect_ratio: (1.).into(),
                    margin: UiRect::left(Val::Px(5.)),
                    ..default()
                },
                ..default()
            },
        )
    }
}

pub trait UiMenuItemExt<'w, 's> {
    fn menu_item<'a>(&'a mut self, config: MenuItemConfig) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiMenuItemExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn menu_item<'a>(&'a mut self, config: MenuItemConfig) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut leading = Entity::PLACEHOLDER;
        let mut label = Entity::PLACEHOLDER;
        let mut shortcut = Entity::PLACEHOLDER;
        let mut trailing = Entity::PLACEHOLDER;
        let name = format!("Menu Item [{}]", config.name.clone());

        let mut item = self.container((Name::new(name), MenuItem::button(), config), |menu_item| {
            leading = menu_item.spawn(MenuItem::leading_icon()).id();
            label = menu_item
                .label(LabelConfig {
                    label: "".into(),
                    margin: UiRect::horizontal(Val::Px(5.)),
                    color: Color::ANTIQUE_WHITE,
                    ..default()
                })
                .id();
            menu_item.container(MenuItem::shortcut(), |shortcut_container| {
                shortcut = shortcut_container
                    .label(LabelConfig {
                        label: "".into(),
                        margin: UiRect::horizontal(Val::Px(5.)),
                        color: Color::ANTIQUE_WHITE,
                        ..default()
                    })
                    .id();
            });

            trailing = menu_item.spawn(MenuItem::trailing_icon()).id();
        });

        item.insert(MenuItem {
            leading,
            label,
            shortcut,
            trailing,
            ..default()
        });

        item
    }
}
