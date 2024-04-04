use bevy::prelude::*;
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    ui_builder::*,
    ui_style::{SetBorderColorExt, SetVisibilityExt, UiStyleExt},
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use super::prelude::{LabelConfig, MenuItem, UiContainerExt, UiLabelExt};

const MENU_CONTAINER_Z_INDEX: i32 = 100000;

// TODO: Implement scrolling and up/down arrows when menu too large (>70%?)
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, MenuUpdate.after(FluxInteractionUpdate))
            .add_systems(
                Update,
                (
                    handle_click_or_touch,
                    handle_item_interaction,
                    update_menu_container_visibility,
                )
                    .chain()
                    .in_set(MenuUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct MenuUpdate;

fn handle_click_or_touch(
    r_mouse: Res<ButtonInput<MouseButton>>,
    r_touches: Res<Touches>,
    q_menu_items: Query<(&MenuItem, Ref<FluxInteraction>)>,
    mut q_menus: Query<(Entity, &mut Menu, Ref<FluxInteraction>)>,
) {
    if r_mouse.any_just_pressed([MouseButton::Left, MouseButton::Middle, MouseButton::Right])
        || r_touches.any_just_pressed()
    {
        let any_pressed = q_menus
            .iter()
            .any(|(_, _, f)| *f == FluxInteraction::Pressed);
        if !any_pressed {
            for (_, interaction) in &q_menu_items {
                if interaction.is_changed() && *interaction == FluxInteraction::Pressed {
                    return;
                }
            }

            for (_, mut menu, _) in &mut q_menus {
                menu.is_open = false;
            }
            return;
        }
    }

    if r_mouse.any_just_released([MouseButton::Left, MouseButton::Middle, MouseButton::Right])
        || r_touches.any_just_released()
    {
        let any_pressed = q_menus
            .iter()
            .any(|(_, _, f)| *f == FluxInteraction::Released);
        if !any_pressed {
            for (_, mut menu, _) in &mut q_menus {
                menu.is_open = false;
            }
            return;
        }
    }

    let any_changed = q_menus.iter().any(|(_, _, f)| f.is_changed());
    if !any_changed {
        return;
    }

    let any_open = q_menus.iter().any(|(_, m, _)| m.is_open);
    let mut open: Option<Entity> =
        if let Some((entity, _, _)) = q_menus.iter().find(|(_, m, _)| m.is_open) {
            entity.into()
        } else {
            None
        };

    for (entity, menu, interaction) in &mut q_menus {
        if interaction.is_changed() {
            if (menu.is_open && *interaction == FluxInteraction::Pressed)
                || (!menu.is_open && *interaction == FluxInteraction::Released)
            {
                open = None;
                break;
            }
            if *interaction == FluxInteraction::Pressed || *interaction == FluxInteraction::Released
            {
                open = entity.into();
                break;
            }
            if any_open && *interaction == FluxInteraction::PointerEnter {
                open = entity.into();
                break;
            }
        }
    }

    for (entity, mut menu, _) in &mut q_menus {
        if let Some(open_dropdown) = open {
            if entity == open_dropdown {
                if !menu.is_open {
                    menu.is_open = true;
                }
            } else if menu.is_open {
                menu.is_open = false;
            }
        } else if menu.is_open {
            menu.is_open = false;
        }
    }
}

fn handle_item_interaction(
    q_menu_items: Query<&MenuItem, Changed<MenuItem>>,
    mut q_menus: Query<&mut Menu>,
) {
    let any_interacted = q_menu_items.iter().any(|item| item.interacted());
    if any_interacted {
        for mut menu in &mut q_menus {
            menu.is_open = false;
        }
    }
}

fn update_menu_container_visibility(
    q_menus: Query<(Entity, &Menu), Changed<Menu>>,
    mut commands: Commands,
) {
    for (entity, menu) in &q_menus {
        commands
            .style(menu.container)
            .visibility(match menu.is_open {
                true => Visibility::Inherited,
                false => Visibility::Hidden,
            });

        commands.style(entity).border_color(match menu.is_open {
            true => Color::ANTIQUE_WHITE,
            false => Color::NONE,
        });
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MenuConfig {
    pub name: String,
    pub alt_code: Option<KeyCode>,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Menu {
    container: Entity,
    is_open: bool,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
            is_open: false,
        }
    }
}

impl Menu {
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
                    padding: UiRect::axes(Val::Px(10.), Val::Px(5.)),
                    border: UiRect::horizontal(Val::Px(1.)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::NONE.into(),
                border_color: Color::NONE.into(),
                ..default()
            },
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Color::DARK_GRAY.into(),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: Menu::base_tween(),
                ..default()
            },
        )
    }

    fn container() -> impl Bundle {
        (
            Name::new("Container"),
            NodeBundle {
                style: Style {
                    top: Val::Px(22.),
                    left: Val::Px(-1.),
                    position_type: PositionType::Absolute,
                    border: UiRect::px(1., 1., 0., 1.),
                    padding: UiRect::px(5., 5., 5., 10.),
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::End,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                z_index: ZIndex::Global(MENU_CONTAINER_Z_INDEX),
                background_color: Color::rgb(0.1, 0.1, 0.1).into(),
                border_color: Color::ANTIQUE_WHITE.into(),
                focus_policy: bevy::ui::FocusPolicy::Block,
                visibility: Visibility::Hidden,
                ..default()
            },
            Interaction::default(),
        )
    }
}

pub trait UiMenuExt<'w, 's> {
    fn menu<'a>(
        &'a mut self,
        config: MenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiMenuExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn menu<'a>(
        &'a mut self,
        config: MenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut container = Entity::PLACEHOLDER;
        let name = format!("Menu [{}]", config.name.clone());

        let mut menu = self.container((Name::new(name), Menu::button()), |menu_button| {
            container = menu_button.container(Menu::container(), spawn_items).id();
            menu_button.label(LabelConfig {
                label: config.name.clone(),
                color: Color::ANTIQUE_WHITE,
                ..default()
            });
        });

        menu.insert((
            Menu {
                container,
                ..default()
            },
            config,
        ));

        menu
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MenuSeparator;

impl MenuSeparator {
    fn separator() -> impl Bundle {
        NodeBundle {
            style: Style {
                height: Val::Px(12.),
                width: Val::Px(1.),
                margin: UiRect::horizontal(Val::Px(5.)),
                ..default()
            },
            background_color: Color::ANTIQUE_WHITE.into(),
            ..default()
        }
    }
}

pub trait UiMenuSeparatorExt<'w, 's> {
    fn menu_separator<'a>(&'a mut self) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiMenuSeparatorExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn menu_separator<'a>(&'a mut self) -> UiBuilder<'w, 's, 'a, Entity> {
        self.spawn(MenuSeparator::separator())
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MenuItemSeparator;

impl MenuItemSeparator {
    fn separator() -> impl Bundle {
        NodeBundle {
            style: Style {
                min_width: Val::Px(100.),
                height: Val::Px(1.),
                margin: UiRect::px(5., 5., 5., 5.),
                ..default()
            },
            background_color: Color::ANTIQUE_WHITE.into(),
            ..default()
        }
    }
}

pub trait UiMenuItemSeparatorExt<'w, 's> {
    fn menu_item_separator<'a>(&'a mut self) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiMenuItemSeparatorExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn menu_item_separator<'a>(&'a mut self) -> UiBuilder<'w, 's, 'a, Entity> {
        self.spawn(MenuItemSeparator::separator())
    }
}
