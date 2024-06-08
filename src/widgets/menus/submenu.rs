use bevy::{prelude::*, ui::FocusPolicy};

use sickle_ui_scaffold::{
    theme::prelude::*,
    ui_builder::{UiBuilder, UiBuilderExt},
    ui_commands::ManagePseudoStateExt,
    ui_style::{LockableStyleAttribute, LockedStyleAttributes, StyleBuilder},
    FluxInteraction, FluxInteractionStopwatch, FluxInteractionUpdate, TrackedInteraction,
};

use super::{
    context_menu::{ContextMenu, ContextMenuUpdate, UiContextMenuExt},
    menu::{Menu, MenuUpdate, UiMenuSubExt},
    menu_item::{MenuItem, MenuItemConfig},
};

const MENU_CONTAINER_FADE_TIMEOUT: f32 = 1.;
const MENU_CONTAINER_SWITCH_TIMEOUT: f32 = 0.3;

// TODO: Add vertically scrollable container and height constraint
// TODO: Best effort position submenu within window bounds
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
        .add_plugins(ComponentThemePlugin::<Submenu>::default())
        .add_systems(
            Update,
            (
                unlock_submenu_container_on_menu_interaction,
                update_submenu_timeout,
                open_submenu_on_hover,
                close_submenus_on_menu_change,
                update_open_submenu_containers,
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

        let Ok(interaction) = q_external_interaction.get(container.external_container) else {
            continue;
        };

        if interaction.is_changed() {
            state.is_locked = false;
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
    let mut opened: Option<(Entity, Entity)> = None;
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
            open_external = container.external_container.into();
        } else {
            containers_closed.push(entity);
        }
    }

    if let (Some(open), Some(external)) = (open_container, open_external) {
        for (entity, mut container) in q_all_containers.iter_mut(world) {
            if container.external_container == external && container.is_open && entity != open {
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

fn update_submenu_style(
    q_submenus: Query<(Entity, &Submenu), Changed<Submenu>>,
    mut commands: Commands,
) {
    for (entity, submenu) in &q_submenus {
        if submenu.is_open {
            commands.entity(entity).add_pseudo_state(PseudoState::Open);
        } else {
            commands
                .entity(entity)
                .remove_pseudo_state(PseudoState::Open);
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

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct SubmenuContainer {
    is_open: bool,
    external_container: Entity,
}

impl Default for SubmenuContainer {
    fn default() -> Self {
        Self {
            is_open: Default::default(),
            external_container: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubmenuConfig {
    pub name: String,
    pub alt_code: Option<KeyCode>,
    pub leading_icon: IconData,
}

impl Into<MenuItemConfig> for SubmenuConfig {
    fn into(self) -> MenuItemConfig {
        MenuItemConfig {
            name: self.name,
            alt_code: self.alt_code,
            leading_icon: self.leading_icon,
            ..default()
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct Submenu {
    is_open: bool,
    is_focused: bool,
    container: Entity,
    external_container: Entity,
    leading: Entity,
    leading_icon: IconData,
    label: Entity,
    shortcut_container: Entity,
    shortcut: Entity,
    trailing: Entity,
    alt_code: Option<KeyCode>,
}

impl Default for Submenu {
    fn default() -> Self {
        Self {
            is_open: false,
            is_focused: false,
            container: Entity::PLACEHOLDER,
            external_container: Entity::PLACEHOLDER,
            leading: Entity::PLACEHOLDER,
            leading_icon: Default::default(),
            label: Entity::PLACEHOLDER,
            shortcut_container: Entity::PLACEHOLDER,
            shortcut: Entity::PLACEHOLDER,
            trailing: Entity::PLACEHOLDER,
            alt_code: Default::default(),
        }
    }
}

impl Into<Submenu> for MenuItem {
    fn into(self) -> Submenu {
        Submenu {
            is_open: false,
            is_focused: false,
            external_container: Entity::PLACEHOLDER,
            container: Entity::PLACEHOLDER,
            label: self.label(),
            leading: self.leading(),
            leading_icon: self.leading_icon(),
            shortcut_container: self.shortcut_container(),
            shortcut: self.shortcut(),
            trailing: self.trailing(),
            alt_code: self.alt_code(),
        }
    }
}

impl DefaultTheme for Submenu {
    fn default_theme() -> Option<Theme<Submenu>> {
        Submenu::theme().into()
    }
}

impl UiContext for Submenu {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            MenuItem::LEADING_ICON => Ok(self.leading),
            MenuItem::LABEL => Ok(self.label),
            MenuItem::SHORTCUT_CONTAINER => Ok(self.shortcut_container),
            MenuItem::SHORTCUT => Ok(self.shortcut),
            MenuItem::TRAILING_ICON => Ok(self.trailing),
            Submenu::MENU_CONTAINER => Ok(self.container),
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
            Submenu::MENU_CONTAINER,
        ]
    }
}

impl Submenu {
    pub const MENU_CONTAINER: &'static str = "MenuContainer";

    pub fn theme() -> Theme<Submenu> {
        let base_theme = PseudoTheme::deferred_world(None, Submenu::primary_style);
        let open_theme = PseudoTheme::deferred_world(vec![PseudoState::Open], Submenu::open_style);

        Theme::<Submenu>::new(vec![base_theme, open_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, entity: Entity, world: &mut World) {
        let Some(menu_item) = world.get::<Submenu>(entity) else {
            return;
        };

        let theme_data = world.resource::<ThemeData>().clone();
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let leading_icon = menu_item.leading_icon.clone();
        let trailing_icon = theme_data.icons.arrow_right;

        MenuItem::menu_item_style(style_builder, world, leading_icon, trailing_icon);

        style_builder
            .switch_target(Submenu::MENU_CONTAINER)
            .position_type(PositionType::Absolute)
            .top(Val::Px(0.))
            .border(UiRect::all(Val::Px(theme_spacing.borders.extra_small)))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .flex_direction(FlexDirection::Column)
            .z_index(ZIndex::Local(1))
            .background_color(colors.container(Container::SurfaceHigh))
            .border_color(colors.accent(Accent::Shadow))
            .display(Display::None)
            .visibility(Visibility::Hidden);
    }

    fn open_style(style_builder: &mut StyleBuilder, entity: Entity, world: &mut World) {
        let theme_data = world.resource::<ThemeData>().clone();
        let colors = theme_data.colors();

        style_builder.background_color(colors.accent(Accent::OutlineVariant));

        // Unsafe unwrap: if the menu item doesn't have a node, panic!
        let node = world.get::<Node>(entity).unwrap();

        style_builder
            .switch_target(Submenu::MENU_CONTAINER)
            .left(Val::Px(node.size().x))
            .display(Display::Flex)
            .visibility(Visibility::Inherited);
    }

    fn container_bundle(external_container: Entity) -> impl Bundle {
        (
            Name::new("Submenu Container"),
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
            TrackedInteraction::default(),
            SubmenuContainerState::default(),
            SubmenuContainer {
                external_container,
                ..default()
            },
        )
    }
}

pub trait UiSubmenuSubExt<'w, 's> {
    fn container(&self) -> Entity;
}

impl<'w, 's> UiSubmenuSubExt<'w, 's> for UiBuilder<'w, 's, '_, Submenu> {
    fn container(&self) -> Entity {
        self.context().container
    }
}

pub trait UiSubmenuExt<'w, 's> {
    fn submenu<'a>(
        &'a mut self,
        config: SubmenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder<Submenu>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiSubmenuExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn submenu<'a>(
        &'a mut self,
        config: SubmenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder<Submenu>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let external_container = self.id();
        let (id, menu_item) = MenuItem::scaffold(self, config.into());
        let container = self
            .commands()
            .ui_builder(id)
            .spawn(Submenu::container_bundle(external_container))
            .id();

        let submenu = Submenu {
            container,
            external_container,
            ..menu_item.into()
        };

        let mut content_builder = self.commands().ui_builder(submenu.clone());
        spawn_items(&mut content_builder);

        self.commands().ui_builder(id).insert(submenu);
        self.commands().ui_builder(id)
    }
}

impl<'w, 's> UiSubmenuExt<'w, 's> for UiBuilder<'w, 's, '_, Menu> {
    fn submenu<'a>(
        &'a mut self,
        config: SubmenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder<Submenu>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let container_id = self.container();
        let id = self
            .commands()
            .ui_builder(container_id)
            .submenu(config, spawn_items)
            .id();

        self.commands().ui_builder(id)
    }
}

impl<'w, 's> UiSubmenuExt<'w, 's> for UiBuilder<'w, 's, '_, Submenu> {
    fn submenu<'a>(
        &'a mut self,
        config: SubmenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder<Submenu>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let container_id = self.container();
        let id = self
            .commands()
            .ui_builder(container_id)
            .submenu(config, spawn_items)
            .id();

        self.commands().ui_builder(id)
    }
}

impl<'w, 's> UiSubmenuExt<'w, 's> for UiBuilder<'w, 's, '_, ContextMenu> {
    fn submenu<'a>(
        &'a mut self,
        config: SubmenuConfig,
        spawn_items: impl FnOnce(&mut UiBuilder<Submenu>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let container_id = self.container();
        let id = self
            .commands()
            .ui_builder(container_id)
            .submenu(config, spawn_items)
            .id();

        self.commands().ui_builder(id)
    }
}
