use bevy::{ecs::system::Command, prelude::*};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    drag_interaction::{DragState, Draggable, DraggableUpdate},
    interactions::InteractiveBackground,
    ui_builder::{UiBuilder, UiBuilderExt, UiContextRoot},
    ui_style::{
        SetBackgroundColorExt, SetFluxInteractionExt, SetNodeLeftExt, SetNodeOverflowExt,
        SetNodePositionTypeExt, SetNodeShowHideExt, SetZIndexExt, UiStyleExt,
    },
    TrackedInteraction,
};

use super::{
    context_menu::ContextMenuUpdate,
    floating_panel::{FloatingPanel, FloatingPanelUpdate, UpdateFloatingPanelPanelId},
    panel::Panel,
    prelude::{
        ContextMenuGenerator, FloatingPanelConfig, FloatingPanelLayout, GenerateContextMenu,
        LabelConfig, MenuItem, MenuItemConfig, MenuItemUpdate, ReflectContextMenuGenerator,
        UiContainerExt, UiFloatingPanelExt, UiLabelExt, UiMenuItemExt, UiPanelExt, UiScrollViewExt,
    },
    sized_zone::SizedZonePreUpdate,
};

pub struct TabContainerPlugin;

impl Plugin for TabContainerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            TabContainerUpdate
                .after(DraggableUpdate)
                .before(FloatingPanelUpdate),
        )
        .register_type::<Tab>()
        .add_systems(
            PreUpdate,
            (
                dock_panel_in_tab_container,
                popout_panel_from_tab.before(SizedZonePreUpdate),
            ),
        )
        .add_systems(
            Update,
            (
                close_tab_on_context_menu_press,
                popout_tab_on_context_menu_press,
            )
                .after(MenuItemUpdate)
                .before(ContextMenuUpdate)
                .before(TabContainerUpdate),
        )
        .add_systems(
            Update,
            (
                update_tab_container_on_tab_press,
                update_tab_container_on_change,
                handle_tab_dragging,
            )
                .chain()
                .in_set(TabContainerUpdate),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct TabContainerUpdate;

fn dock_panel_in_tab_container(
    mut q_docking_panels: Query<
        (Entity, &mut TabContainer, &DockFloatingPanel),
        Added<DockFloatingPanel>,
    >,
    q_floating_panel: Query<&FloatingPanel>,
    q_panel: Query<&Panel>,
    mut commands: Commands,
) {
    for (container_id, mut tab_container, dock_ref) in &mut q_docking_panels {
        commands.entity(container_id).remove::<DockFloatingPanel>();

        let Ok(floating_panel) = q_floating_panel.get(dock_ref.floating_panel) else {
            warn!(
                "Failed to dock floating panel {:?}: Not a FloatingPanel",
                dock_ref.floating_panel
            );
            continue;
        };

        let panel_id = floating_panel.content_panel_id();

        let Ok(panel) = q_panel.get(panel_id) else {
            warn!(
                "Failed to dock floating panel {:?}: Missing Panel {:?}",
                dock_ref.floating_panel, panel_id
            );
            continue;
        };

        let bar_id = tab_container.bar;
        let viewport_id = tab_container.viewport;

        commands.ui_builder(bar_id).container(
            (
                TabContainer::tab(),
                Tab {
                    container: container_id,
                    bar: bar_id,
                    panel: panel_id,
                    ..default()
                },
            ),
            |container| {
                container.label(LabelConfig {
                    label: panel.title(),
                    ..default()
                });
            },
        );

        commands.entity(viewport_id).add_child(panel_id);
        commands.entity(dock_ref.floating_panel).despawn_recursive();
        commands.style(panel_id).hide();

        tab_container.tab_count += 1;
        tab_container.active = tab_container.tab_count - 1;
    }
}

fn popout_panel_from_tab(
    q_popout: Query<
        (Entity, &Tab, &PopoutPanelFromTabContainer),
        Added<PopoutPanelFromTabContainer>,
    >,
    q_panel: Query<&Panel>,
    q_parent: Query<&Parent>,
    q_ui_context_root: Query<&UiContextRoot>,
    mut q_tab_container: Query<&mut TabContainer>,
    mut commands: Commands,
) {
    for (entity, tab, popout_ref) in &q_popout {
        commands
            .entity(entity)
            .remove::<PopoutPanelFromTabContainer>();

        let tab_contaier_id = tab.container;

        let Ok(mut tab_container) = q_tab_container.get_mut(tab_contaier_id) else {
            warn!(
                "Failed to remove Tab {:?}: {:?} is not a TabContainer!",
                entity, tab_contaier_id,
            );
            continue;
        };
        tab_container.tab_count = match tab_container.tab_count > 1 {
            true => tab_container.tab_count - 1,
            false => 0,
        };

        if tab_container.active >= tab_container.tab_count {
            tab_container.active = match tab_container.tab_count > 0 {
                true => tab_container.tab_count - 1,
                false => 0,
            };
        }

        let panel_id = tab.panel;
        let Ok(panel) = q_panel.get(panel_id) else {
            warn!("Cannot pop out panel {:?}: Not a Panel", panel_id);
            continue;
        };
        let title = panel.title();

        let root_node = q_parent
            .iter_ancestors(tab_contaier_id)
            .find(|parent| q_ui_context_root.get(*parent).is_ok())
            .or(q_parent.iter_ancestors(tab_contaier_id).last())
            .unwrap_or(tab_contaier_id);

        commands.entity(entity).despawn_recursive();
        let floating_panel_id = commands
            .ui_builder(root_node)
            .floating_panel(
                FloatingPanelConfig {
                    title: title.into(),
                    ..default()
                },
                FloatingPanelLayout {
                    size: popout_ref.size,
                    position: popout_ref.position.into(),
                    droppable: true,
                    ..default()
                },
                |_| {},
            )
            .id();

        commands.entity(panel_id).set_parent(root_node);
        commands.style(panel_id).hide();
        commands
            .entity(floating_panel_id)
            .insert(UpdateFloatingPanelPanelId { panel_id });
    }
}

fn close_tab_on_context_menu_press(
    q_menu_items: Query<(Entity, &CloseTabContextMenu, &MenuItem), Changed<MenuItem>>,
    q_tab: Query<&Tab>,
    mut q_tab_container: Query<&mut TabContainer>,
    mut commands: Commands,
) {
    for (entity, context_menu, menu_item) in &q_menu_items {
        if menu_item.interacted() {
            let Ok(tab_data) = q_tab.get(context_menu.tab) else {
                warn!(
                    "Context menu {:?} refers to missing tab {:?}",
                    entity, context_menu.tab
                );
                continue;
            };

            let tab_contaier_id = tab_data.container;
            let Ok(mut tab_container) = q_tab_container.get_mut(tab_contaier_id) else {
                warn!(
                    "Failed to remove Tab {:?}: {:?} is not a TabContainer!",
                    entity, tab_contaier_id,
                );
                continue;
            };
            tab_container.tab_count = match tab_container.tab_count > 1 {
                true => tab_container.tab_count - 1,
                false => 0,
            };

            commands.entity(context_menu.tab).despawn_recursive();
            commands.entity(tab_data.panel).despawn_recursive();
        }
    }
}

fn popout_tab_on_context_menu_press(
    q_menu_items: Query<(Entity, &PopoutTabContextMenu, &MenuItem), Changed<MenuItem>>,
    q_tab: Query<(&Tab, &GlobalTransform)>,
    q_node: Query<&Node>,
    mut commands: Commands,
) {
    for (entity, tab_ref, menu_item) in &q_menu_items {
        if menu_item.interacted() {
            let Ok((tab, transform)) = q_tab.get(tab_ref.tab) else {
                warn!(
                    "Context menu tab reference {:?} refers to missing tab {:?}",
                    entity, tab_ref.tab
                );
                continue;
            };

            let Ok(container) = q_node.get(tab.container) else {
                warn!(
                    "Context menu tab reference {:?} refers to a tab without a container {:?}",
                    entity, tab_ref.tab
                );
                continue;
            };

            let size = container.size() * 0.8;
            let position = transform.translation().truncate();
            commands
                .entity(tab_ref.tab)
                .insert(PopoutPanelFromTabContainer { size, position });
        }
    }
}

fn update_tab_container_on_tab_press(
    q_tabs: Query<(Entity, &Tab, &Interaction), Changed<Interaction>>,
    q_tab: Query<Entity, With<Tab>>,
    q_children: Query<&Children>,
    mut q_tab_container: Query<&mut TabContainer>,
) {
    for (tab_entity, tab, interaction) in &q_tabs {
        if *interaction == Interaction::Pressed {
            let Ok(mut tab_container) = q_tab_container.get_mut(tab.container) else {
                continue;
            };

            let Ok(tabs) = q_children.get(tab_container.bar) else {
                continue;
            };

            for (i, id) in tabs.iter().enumerate() {
                if let Ok(_) = q_tab.get(*id) {
                    if *id == tab_entity {
                        tab_container.active = i;
                    }
                }
            }
        }
    }
}

fn update_tab_container_on_change(
    q_tab_containers: Query<&TabContainer, Changed<TabContainer>>,
    q_tab: Query<(Entity, &Tab), With<Tab>>,
    q_children: Query<&Children>,
    mut commands: Commands,
) {
    for tab_container in &q_tab_containers {
        let Ok(tabs) = q_children.get(tab_container.bar) else {
            continue;
        };

        let flux_enabled = tabs.iter().filter(|tab| q_tab.get(**tab).is_ok()).count() > 1;
        for (i, id) in tabs.iter().enumerate() {
            if let Ok((tab_entity, tab)) = q_tab.get(*id) {
                commands
                    .style(tab_entity)
                    .flux_interaction_enabled(flux_enabled);

                if i == tab_container.active {
                    commands.style(tab_entity).background_color(Color::GRAY);
                    commands.style(tab.panel).show();
                } else {
                    commands.style(tab_entity).background_color(Color::NONE);
                    commands.style(tab.panel).hide();
                }
            }
        }
    }
}

fn handle_tab_dragging(
    q_tabs: Query<(Entity, &Draggable, &Node, &Transform), (With<Tab>, Changed<Draggable>)>,
    q_tab_container: Query<&TabContainer>,
    q_tab_bar: Query<&Node, With<TabBar>>,
    q_children: Query<&Children>,
    q_transform: Query<(&GlobalTransform, &Interaction)>,
    mut q_tab: Query<&mut Tab>,
    mut commands: Commands,
) {
    for (entity, draggable, node, transform) in &q_tabs {
        let tab = q_tab.get(entity).unwrap();

        let Ok(container) = q_tab_container.get(tab.container) else {
            warn!("Tried to drag orphan Tab {:?}", entity);
            continue;
        };

        let Ok(bar_node) = q_tab_bar.get(container.bar) else {
            error!("Tab container {:?} doesn't have a tab bar", tab.container);
            continue;
        };

        let Ok(children) = q_children.get(container.bar) else {
            error!("Tab container has no tabs {:?}", tab.container);
            continue;
        };

        if children
            .iter()
            .filter(|child| q_tab.get(**child).is_ok())
            .count()
            < 2
        {
            continue;
        }

        let bar_half_width = bar_node.size().x / 2.;
        match draggable.state {
            DragState::DragStart => {
                commands.style(container.bar).overflow(Overflow::visible());

                children.iter().for_each(|child| {
                    if *child != entity && q_tab.get(*child).is_ok() {
                        commands.style(*child).disable_flux_interaction();
                    }
                });

                let Some(tab_index) = children
                    .iter()
                    .filter(|child| q_tab.get(**child).is_ok())
                    .position(|child| *child == entity)
                else {
                    error!("Tab {:?} isn't a child of its tab container bar", entity);
                    continue;
                };

                let left =
                    transform.translation.truncate().x - (node.size().x / 2.) + bar_half_width;
                let placeholder = commands
                    .ui_builder(container.bar)
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(node.size().x * 1.1),
                            height: Val::Px(node.size().y),
                            ..default()
                        },
                        background_color: Color::NAVY.into(),
                        ..default()
                    })
                    .id();

                commands
                    .entity(container.bar)
                    .insert_children(tab_index, &[placeholder]);

                commands
                    .ui_builder(entity)
                    .style()
                    .position_type(PositionType::Absolute)
                    .left(Val::Px(left))
                    .z_index(ZIndex::Local(100));

                let mut tab = q_tab.get_mut(entity).unwrap();
                tab.placeholder = placeholder.into();
                tab.original_index = tab_index.into();
            }
            DragState::Dragging => {
                let Some(diff) = draggable.diff else {
                    continue;
                };
                let Some(position) = draggable.position else {
                    continue;
                };

                let Some(placeholder) = tab.placeholder else {
                    warn!("Tab {:?} missing placeholder", entity);
                    continue;
                };

                let new_x = transform.translation.truncate().x + diff.x + bar_half_width;
                let left = new_x - (node.size().x / 2.);
                let mut new_index: Option<usize> = None;
                let mut placeholder_index = children.len();
                for (i, child) in children.iter().enumerate() {
                    if *child == entity {
                        continue;
                    }
                    if *child == placeholder {
                        placeholder_index = i;
                        continue;
                    }
                    let Ok(_) = q_tab.get(entity) else {
                        continue;
                    };
                    let Ok((transform, interaction)) = q_transform.get(*child) else {
                        continue;
                    };

                    if *interaction == Interaction::Hovered {
                        if position.x < transform.translation().truncate().x {
                            if i < placeholder_index {
                                new_index = i.into();
                            } else {
                                // placeholder is between 0 and children.len or less
                                new_index = (i - 1).into();
                            }
                        } else {
                            if i + 1 < placeholder_index {
                                new_index = (i + 1).into();
                            } else {
                                // placeholder is between 0 and children.len or less
                                new_index = i.into();
                            }
                        }

                        break;
                    }
                }

                if let Some(new_index) = new_index {
                    commands
                        .entity(container.bar)
                        .insert_children(new_index, &[placeholder]);
                }

                commands.ui_builder(entity).style().left(Val::Px(left));
            }
            DragState::DragEnd => {
                commands.style(container.bar).overflow(Overflow::clip());

                children.iter().for_each(|child| {
                    if *child != entity && q_tab.get(*child).is_ok() {
                        commands.style(*child).enable_flux_interaction();
                    }
                });

                let Some(placeholder) = tab.placeholder else {
                    warn!("Tab {:?} missing placeholder", entity);
                    continue;
                };

                let Some(placeholder_index) =
                    children.iter().position(|child| *child == placeholder)
                else {
                    error!(
                        "Tab placeholder {:?} isn't a child of its tab container bar",
                        entity
                    );
                    continue;
                };

                commands
                    .style(entity)
                    .position_type(PositionType::Relative)
                    .left(Val::Auto)
                    .z_index(ZIndex::Local(0));

                commands
                    .entity(container.bar)
                    .insert_children(placeholder_index, &[entity]);

                commands.entity(placeholder).despawn_recursive();

                let mut tab = q_tab.get_mut(entity).unwrap();
                tab.placeholder = None;
                tab.original_index = None;
            }
            DragState::DragCanceled => {
                commands.style(container.bar).overflow(Overflow::clip());

                children.iter().for_each(|child| {
                    if *child != entity && q_tab.get(*child).is_ok() {
                        commands.style(*child).enable_flux_interaction();
                    }
                });

                let Some(placeholder) = tab.placeholder else {
                    warn!("Tab {:?} missing placeholder", entity);
                    continue;
                };

                let original_index = tab.original_index.unwrap_or(0);

                commands
                    .style(entity)
                    .position_type(PositionType::Relative)
                    .left(Val::Auto)
                    .z_index(ZIndex::Local(0));

                commands.entity(placeholder).despawn_recursive();

                commands
                    .entity(container.bar)
                    .insert_children(original_index, &[entity]);

                let mut tab = q_tab.get_mut(entity).unwrap();
                tab.placeholder = None;
                tab.original_index = None;
            }
            _ => continue,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CloseTabContextMenu {
    tab: Entity,
}

impl Default for CloseTabContextMenu {
    fn default() -> Self {
        Self {
            tab: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PopoutTabContextMenu {
    tab: Entity,
}

impl Default for PopoutTabContextMenu {
    fn default() -> Self {
        Self {
            tab: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component, ContextMenuGenerator)]
pub struct Tab {
    container: Entity,
    bar: Entity,
    panel: Entity,
    placeholder: Option<Entity>,
    original_index: Option<usize>,
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
            bar: Entity::PLACEHOLDER,
            panel: Entity::PLACEHOLDER,
            placeholder: None,
            original_index: None,
        }
    }
}

impl ContextMenuGenerator for Tab {
    fn build_context_menu(&self, context: Entity, container: &mut UiBuilder<Entity>) {
        container
            .menu_item(MenuItemConfig {
                name: "Close Tab".into(),
                leading_icon: Some("sickle_ui://icons/close.png".into()),
                ..default()
            })
            .insert(CloseTabContextMenu { tab: context });
        container
            .menu_item(MenuItemConfig {
                name: "Popout Tab".into(),
                trailing_icon: Some("sickle_ui://icons/popout_white.png".into()),
                ..default()
            })
            .insert(PopoutTabContextMenu { tab: context });
    }

    fn placement_index(&self) -> usize {
        0
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct PopoutPanelFromTabContainer {
    size: Vec2,
    position: Vec2,
}

struct IncrementTabCount {
    container: Entity,
}

impl Command for IncrementTabCount {
    fn apply(self, world: &mut World) {
        let Some(mut container) = world.get_mut::<TabContainer>(self.container) else {
            warn!(
                "Failed to increment tab count: {:?} is not a TabContainer!",
                self.container,
            );
            return;
        };

        container.tab_count += 1;
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct DockFloatingPanel {
    floating_panel: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct TabBar {
    container: Entity,
}

impl Default for TabBar {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
        }
    }
}

impl TabBar {
    pub fn container_id(&self) -> Entity {
        self.container
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct TabViewport {
    container: Entity,
}

impl Default for TabViewport {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct TabContainer {
    own_id: Entity,
    active: usize,
    bar: Entity,
    viewport: Entity,
    tab_count: usize,
}

impl Default for TabContainer {
    fn default() -> Self {
        Self {
            own_id: Entity::PLACEHOLDER,
            active: 0,
            tab_count: 0,
            bar: Entity::PLACEHOLDER,
            viewport: Entity::PLACEHOLDER,
        }
    }
}

impl TabContainer {
    pub fn bar_id(&self) -> Entity {
        self.bar
    }

    pub fn tab_count(&self) -> usize {
        self.tab_count
    }

    pub fn set_active(&mut self, active: usize) {
        self.active = active;
    }
}

impl TabContainer {
    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn frame() -> impl Bundle {
        (
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            Interaction::default(),
        )
    }

    fn bar() -> impl Bundle {
        (
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Px(30.),
                    border: UiRect::bottom(Val::Px(1.)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                border_color: Color::DARK_GRAY.into(),
                ..default()
            },
            Interaction::default(),
        )
    }

    fn tab() -> impl Bundle {
        (
            NodeBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(10.), Val::Px(5.)),
                    border: UiRect::horizontal(Val::Px(1.)),
                    ..default()
                },
                border_color: Color::DARK_GRAY.into(),
                ..default()
            },
            Interaction::default(),
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Color::rgba(0.9, 0.8, 0.7, 0.5).into(),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: TabContainer::base_tween(),
                ..default()
            },
            Draggable::default(),
            GenerateContextMenu::default(),
        )
    }
}

pub trait UiTabContainerExt<'w, 's> {
    fn tab_container<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<TabContainer>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiTabContainerExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn tab_container<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<TabContainer>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut bar = Entity::PLACEHOLDER;
        let mut viewport = Entity::PLACEHOLDER;

        let mut container = self.container(TabContainer::frame(), |container| {
            let container_id = container.id();

            bar = container
                .spawn((
                    TabContainer::bar(),
                    TabBar {
                        container: container_id,
                    },
                ))
                .id();

            container.scroll_view(None, |scroll_view| {
                viewport = scroll_view
                    .insert(TabViewport {
                        container: container_id,
                    })
                    .id();
            });
        });

        let container_id = container.id();
        let tab_container = TabContainer {
            own_id: container_id,
            bar,
            viewport,
            ..default()
        };
        container.insert(tab_container);

        let mut builder = self.commands().ui_builder(tab_container);
        spawn_children(&mut builder);

        self.commands().ui_builder(container_id)
    }
}

pub trait UiTabContainerSubExt<'w, 's> {
    fn id(&self) -> Entity;

    fn add_tab<'a>(
        &'a mut self,
        title: String,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, TabContainer>;

    fn dock_panel<'a>(&'a mut self, floating_panel: Entity) -> UiBuilder<'w, 's, 'a, TabContainer>;
}

impl<'w, 's> UiTabContainerSubExt<'w, 's> for UiBuilder<'w, 's, '_, TabContainer> {
    fn id(&self) -> Entity {
        self.context().own_id
    }

    fn add_tab<'a>(
        &'a mut self,
        title: String,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, TabContainer> {
        let context = self.context();
        let container_id = context.own_id;
        let bar_id = context.bar;
        let viewport_id = context.viewport;
        let panel = self
            .commands()
            .ui_builder(viewport_id)
            .panel(title.clone(), spawn_children)
            .id();

        self.commands().ui_builder(bar_id).container(
            (
                TabContainer::tab(),
                Tab {
                    container: container_id,
                    bar: bar_id,
                    panel,
                    ..default()
                },
            ),
            |container| {
                container.label(LabelConfig {
                    label: title,
                    ..default()
                });
            },
        );

        self.commands().add(IncrementTabCount {
            container: container_id,
        });
        self.commands().ui_builder(context)
    }

    fn dock_panel<'a>(&'a mut self, floating_panel: Entity) -> UiBuilder<'w, 's, 'a, TabContainer> {
        let context = self.context();
        self.commands()
            .entity(context.own_id)
            .insert(DockFloatingPanel { floating_panel });
        self.commands().ui_builder(context)
    }
}
