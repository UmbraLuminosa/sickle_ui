use bevy::prelude::*;

use crate::{
    ui_builder::{UiBuilder, UiBuilderExt},
    ui_style::{SetBackgroundColorExt, SetVisibilityExt, UiStyleExt},
    FluxInteraction, FluxInteractionStopwatch, FluxInteractionUpdate, TrackedInteraction,
};

use super::{
    context_menu::ContextMenuUpdate,
    menu::{Menu, MenuUpdate},
    prelude::{MenuItemConfig, UiContainerExt, UiMenuItemExt},
};

const MENU_CONTAINER_Z_INDEX: i32 = 100001;
const MENU_CONTAINER_FADE_TIMEOUT: f32 = 1.;
const MENU_CONTAINER_SWITCH_TIMEOUT: f32 = 0.3;

// TODO: Add vertically scrollable container and height constraint
// TODO: Best effort position submenu within window bounds
// TODO: Unparent container
pub struct SubmenuPlugin;

impl Plugin for SubmenuPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            SubmenuUpdate
                .after(FluxInteractionUpdate)
                .before(MenuUpdate)
                .before(ContextMenuUpdate),
        )
        .add_systems(
            Update,
            (
                unlock_submenu_container_on_menu_interaction,
                update_submenu_timeout,
                open_submenu_on_hover,
                close_submenus_on_menu_change,
                update_open_submenu_containers,
                update_submenu_container_visibility,
                update_submenu_state,
                update_submenu_style,
            )
                .chain()
                .in_set(SubmenuUpdate),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct SubmenuUpdate;

fn unlock_submenu_container_on_menu_interaction(
    q_external_interaction: Query<Ref<Interaction>>,
    mut q_containers: Query<(&SubmenuContainer, &mut SubmenuContainerState)>,
) {
    for (container, mut state) in &mut q_containers {
        if !container.is_open || !state.is_locked {
            continue;
        }

        if let Some(external_container) = container.external_container {
            let Ok(interaction) = q_external_interaction.get(external_container) else {
                continue;
            };

            if interaction.is_changed() {
                state.is_locked = false;
            }
        }
    }
}

fn update_submenu_timeout(
    r_time: Res<Time>,
    mut q_submenus: Query<(
        &mut SubmenuContainer,
        &mut SubmenuContainerState,
        &FluxInteraction,
    )>,
) {
    for (mut container, mut state, interaction) in &mut q_submenus {
        if *interaction == FluxInteraction::PointerEnter {
            state.is_locked = true;
            state.timeout = MENU_CONTAINER_FADE_TIMEOUT;
        } else if !state.is_locked && state.timeout > 0. {
            state.timeout -= r_time.delta_seconds();
            if container.is_open && state.timeout < 0. {
                container.is_open = false;
            }
        }
    }
}

fn open_submenu_on_hover(
    q_submenus: Query<(
        Entity,
        &Submenu,
        &FluxInteraction,
        &FluxInteractionStopwatch,
    )>,
    mut q_containers: Query<(Entity, &mut SubmenuContainer, &mut SubmenuContainerState)>,
) {
    let mut opened: Option<(Entity, Option<Entity>)> = None;
    for (entity, submenu, interaction, stopwatch) in &q_submenus {
        if *interaction == FluxInteraction::PointerEnter {
            let Ok((entity, mut container, mut state)) = q_containers.get_mut(submenu.container)
            else {
                warn!("Submenu {:?} is missing its container", entity);
                continue;
            };

            if container.is_open {
                continue;
            }

            // Open submenu once hovered enough
            if stopwatch.0.elapsed_secs() > MENU_CONTAINER_SWITCH_TIMEOUT {
                container.is_open = true;
                state.is_locked = true;
                state.timeout = MENU_CONTAINER_FADE_TIMEOUT;

                opened = (entity, container.external_container).into();
            }
        }
    }

    // Force close open siblings after submenu is hovered enough
    if let Some((opened_container, external_container)) = opened {
        for (entity, mut container, mut state) in &mut q_containers {
            if container.is_open
                && container.external_container == external_container
                && entity != opened_container
            {
                container.is_open = false;
                state.is_locked = false;
            }
        }
    }
}

fn close_submenus_on_menu_change(
    q_menus: Query<Entity, Changed<Menu>>,
    mut q_submenus: Query<(&mut SubmenuContainer, &mut SubmenuContainerState)>,
) {
    let any_changed = q_menus.iter().count() > 0;
    if any_changed {
        for (mut container, mut state) in &mut q_submenus {
            container.is_open = false;
            state.is_locked = false;
            state.timeout = 0.;
        }
    }
}

fn update_open_submenu_containers(world: &mut World) {
    let mut q_all_containers = world.query::<(Entity, &mut SubmenuContainer)>();
    let mut q_changed =
        world.query_filtered::<(Entity, &SubmenuContainer), Changed<SubmenuContainer>>();

    let mut containers_closed: Vec<Entity> =
        Vec::with_capacity(q_all_containers.iter(&world).count());
    let mut sibling_containers: Vec<Entity> =
        Vec::with_capacity(q_all_containers.iter(&world).count());
    let mut open_container: Option<Entity> = None;
    let mut open_external: Option<Entity> = None;

    for (entity, container) in q_changed.iter(world) {
        if container.is_open {
            open_container = entity.into();
            open_external = container.external_container;
        } else {
            containers_closed.push(entity);
        }
    }

    if let Some(open) = open_container {
        for (entity, mut container) in q_all_containers.iter_mut(world) {
            if container.external_container == open_external && container.is_open && entity != open
            {
                container.is_open = false;
                sibling_containers.push(entity);
            }
        }
    }

    for entity in sibling_containers.iter() {
        close_containers_of(world, *entity);
    }

    for entity in containers_closed.iter() {
        close_containers_of(world, *entity);
    }
}

fn update_submenu_container_visibility(
    q_submenus: Query<(Entity, &SubmenuContainer), Changed<SubmenuContainer>>,
    mut commands: Commands,
) {
    for (entity, container) in &q_submenus {
        commands.style(entity).visibility(match container.is_open {
            true => Visibility::Inherited,
            false => Visibility::Hidden,
        });
    }
}

fn update_submenu_state(
    mut q_submenus: Query<&mut Submenu>,
    q_submenu_containers: Query<&SubmenuContainer, Changed<SubmenuContainer>>,
) {
    for mut submenu in &mut q_submenus {
        if let Ok(container) = q_submenu_containers.get(submenu.container) {
            if submenu.is_open != container.is_open {
                submenu.is_open = container.is_open;
            }
        }
    }
}

fn update_submenu_style(q_submenus: Query<(Entity, Ref<Submenu>)>, mut commands: Commands) {
    for (entity, submenu) in &q_submenus {
        if submenu.is_open {
            commands.style(entity).background_color(Color::DARK_GRAY);
        } else if submenu.is_changed() {
            commands.style(entity).background_color(Color::NONE);
        }
    }
}

fn close_containers_of(world: &mut World, external: Entity) {
    let mut q_all_containers = world.query::<(Entity, &mut SubmenuContainer)>();
    let mut containers_closed: Vec<Entity> =
        Vec::with_capacity(q_all_containers.iter(&world).count());

    for (entity, mut container) in q_all_containers.iter_mut(world) {
        if container.external_container == external.into() && container.is_open {
            container.is_open = false;
            containers_closed.push(entity);
        }
    }

    for entity in containers_closed.iter() {
        close_containers_of(world, *entity);
    }
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SubmenuContainerState {
    timeout: f32,
    is_locked: bool,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SubmenuContainer {
    is_open: bool,
    external_container: Option<Entity>,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SubmenuConfig {
    pub name: String,
    pub alt_code: Option<KeyCode>,
    pub leading_icon: Option<String>,
}

impl Into<MenuItemConfig> for SubmenuConfig {
    fn into(self) -> MenuItemConfig {
        MenuItemConfig {
            name: self.name,
            alt_code: self.alt_code,
            leading_icon: self.leading_icon,
            trailing_icon: "embedded://sickle_ui/icons/submenu_white.png"
                .to_string()
                .into(),
            ..default()
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Submenu {
    is_open: bool,
    is_focused: bool,
    container: Entity,
    external_container: Option<Entity>,
}

impl Default for Submenu {
    fn default() -> Self {
        Self {
            is_open: false,
            is_focused: false,
            container: Entity::PLACEHOLDER,
            external_container: None,
        }
    }
}

impl SubmenuContainer {
    fn frame() -> impl Bundle {
        (
            NodeBundle {
                style: Style {
                    left: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    border: UiRect::px(1., 1., 1., 1.),
                    padding: UiRect::px(5., 5., 5., 10.),
                    margin: UiRect::px(5., 0., -5., 0.),
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::FlexStart,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                z_index: ZIndex::Global(MENU_CONTAINER_Z_INDEX),
                background_color: Color::rgb(0.1, 0.1, 0.1).into(),
                border_color: Color::ANTIQUE_WHITE.into(),
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            },
            Interaction::default(),
            TrackedInteraction::default(),
        )
    }
}

pub trait UiSubmenuExt<'w, 's> {
    fn submenu<'a>(
        &'a mut self,
        config: SubmenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiSubmenuExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn submenu<'a>(
        &'a mut self,
        config: SubmenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let external_container = Some(self.context());

        let menu_id = self.menu_item(config.clone().into()).id();
        let container = self
            .commands()
            .ui_builder(menu_id)
            .container(
                (
                    Name::new("Submenu Container"),
                    SubmenuContainer::frame(),
                    SubmenuContainerState::default(),
                    SubmenuContainer {
                        external_container,
                        ..default()
                    },
                ),
                spawn_items,
            )
            .id();

        let name = format!("Submenu [{}]", config.name.clone());
        self.commands().ui_builder(menu_id).insert((
            Name::new(name),
            Submenu {
                container,
                external_container,
                ..default()
            },
            config,
        ));

        self.commands().ui_builder(menu_id)
    }
}
