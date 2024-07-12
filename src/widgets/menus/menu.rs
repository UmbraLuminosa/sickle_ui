use bevy::{prelude::*, ui::FocusPolicy};

use sickle_ui_scaffold::prelude::*;

use crate::widgets::layout::{
    container::UiContainerExt,
    label::{LabelConfig, UiLabelExt},
};

use super::{
    menu_bar::{MenuBar, UiMenuBarSubExt},
    menu_item::MenuItem,
};

// TODO: Move all z-index constants to a resource
const MENU_CONTAINER_Z_INDEX: i32 = 100000;

// TODO: Implement scrolling and up/down arrows when menu too large (>70%?)
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, MenuUpdate.after(FluxInteractionUpdate))
            .add_plugins(ComponentThemePlugin::<Menu>::default())
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
        if menu.is_open {
            commands.entity(entity).add_pseudo_state(PseudoState::Open);
        } else {
            commands
                .entity(entity)
                .remove_pseudo_state(PseudoState::Open);
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MenuConfig {
    pub name: String,
    pub alt_code: Option<KeyCode>,
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct Menu {
    label: Entity,
    container: Entity,
    is_open: bool,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            label: Entity::PLACEHOLDER,
            container: Entity::PLACEHOLDER,
            is_open: false,
        }
    }
}

impl UiContext for Menu {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            Menu::LABEL => Ok(self.label),
            Menu::CONTAINER => Ok(self.container),
            _ => Err(format!(
                "{} doesn't exists for Menu. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![Menu::LABEL, Menu::CONTAINER]
    }
}

impl DefaultTheme for Menu {
    fn default_theme() -> Option<Theme<Menu>> {
        Menu::theme().into()
    }
}

impl Menu {
    pub const CONTAINER: &'static str = "Container";
    pub const LABEL: &'static str = "Label";

    pub fn theme() -> Theme<Menu> {
        let base_theme = PseudoTheme::deferred(None, Menu::primary_style);
        let open_theme = PseudoTheme::deferred(vec![PseudoState::Open], Menu::open_style);
        Theme::new(vec![base_theme, open_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);

        style_builder
            .align_items(AlignItems::Center)
            .padding(UiRect::axes(
                Val::Px(theme_spacing.gaps.medium),
                Val::Px(theme_spacing.gaps.small),
            ))
            .border(UiRect::horizontal(Val::Px(
                theme_spacing.borders.extra_small,
            )))
            .border_color(Color::NONE)
            .border_radius(BorderRadius::all(Val::Px(theme_spacing.corners.medium)))
            .animated()
            .background_color(AnimatedVals {
                idle: colors.container(Container::SurfaceMid),
                hover: colors.container(Container::SurfaceHighest).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(Menu::LABEL)
            .sized_font(font)
            .font_color(colors.on(On::Surface));

        style_builder
            .switch_target(Menu::CONTAINER)
            .top(Val::Px(
                theme_spacing.areas.small - theme_spacing.borders.extra_small,
            ))
            .left(Val::Px(-theme_spacing.borders.extra_small))
            .position_type(PositionType::Absolute)
            .border(UiRect::all(Val::Px(theme_spacing.borders.extra_small)))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .flex_direction(FlexDirection::Column)
            .z_index(ZIndex::Global(MENU_CONTAINER_Z_INDEX))
            .background_color(colors.container(Container::SurfaceMid))
            .border_color(colors.accent(Accent::Shadow))
            .border_radius(BorderRadius::all(Val::Px(
                theme_spacing.corners.extra_small,
            )))
            .visibility(Visibility::Hidden);
    }

    fn open_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let colors = theme_data.colors();

        style_builder
            .border_color(colors.accent(Accent::Shadow))
            .background_color(colors.container(Container::SurfaceHighest));

        style_builder
            .switch_target(Menu::LABEL)
            .font_color(colors.on(On::Surface));

        style_builder
            .switch_target(Menu::CONTAINER)
            .visibility(Visibility::Inherited);
    }

    fn button(name: String) -> impl Bundle {
        (
            Name::new(name),
            ButtonBundle::default(),
            TrackedInteraction::default(),
        )
    }

    fn container() -> impl Bundle {
        (
            Name::new("Container"),
            NodeBundle {
                style: Style {
                    overflow: Overflow::visible(),
                    ..default()
                },
                focus_policy: FocusPolicy::Block,
                ..default()
            },
            LockedStyleAttributes::from_vec(vec![
                LockableStyleAttribute::FocusPolicy,
                LockableStyleAttribute::Overflow,
            ]),
            Interaction::default(),
        )
    }
}

pub trait UiMenuExt {
    /// A menu in a MenuBar
    ///
    /// ### PseudoState usage
    /// - `PseudoState::Open` is used when the menu panel is visible
    fn menu(
        &mut self,
        config: MenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder<Menu>),
    ) -> UiBuilder<Entity>;
}

impl UiMenuExt for UiBuilder<'_, Entity> {
    fn menu(
        &mut self,
        config: MenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder<Menu>),
    ) -> UiBuilder<Entity> {
        let mut menu = Menu::default();
        let name = format!("Menu [{}]", config.name.clone());

        let button_id = self
            .container(Menu::button(name), |menu_button| {
                menu.container = menu_button.spawn(Menu::container()).id();
                menu.label = menu_button
                    .label(LabelConfig {
                        label: config.name.clone(),
                        ..default()
                    })
                    .id();
            })
            .insert((menu, config))
            .id();

        let mut menu_builder = self.commands().ui_builder(menu);
        spawn_items(&mut menu_builder);

        self.commands().ui_builder(button_id)
    }
}

impl UiMenuExt for UiBuilder<'_, (Entity, MenuBar)> {
    fn menu(
        &mut self,
        config: MenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder<Menu>),
    ) -> UiBuilder<Entity> {
        let own_id = self.id();
        let id = self
            .commands()
            .ui_builder(own_id)
            .menu(config, spawn_items)
            .id();

        self.commands().ui_builder(id)
    }
}

pub trait UiMenuSubExt {
    fn container(&self) -> Entity;
}

impl UiMenuSubExt for UiBuilder<'_, Menu> {
    fn container(&self) -> Entity {
        self.context().container
    }
}
