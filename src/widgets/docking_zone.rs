use bevy::{
    ecs::system::{Command, CommandQueue},
    prelude::*,
    ui::{FocusPolicy, RelativeCursorPosition},
};
use sickle_macros::UiContext;

use crate::{
    drag_interaction::{DragState, Draggable},
    drop_interaction::{DropPhase, DropZone, DroppableUpdate},
    hierarchy_delay::DelayActions,
    theme::{
        pseudo_state::{PseudoState, PseudoStates},
        theme_colors::Accent,
        theme_data::ThemeData,
        ComponentThemePlugin, PseudoTheme, Theme, UiContext,
    },
    ui_builder::{UiBuilder, UiBuilderExt},
    ui_commands::ResetChildrenInUiSurface,
    ui_style::{
        AnimatedVals, LockableStyleAttribute, LockedStyleAttributes, SetHeightExt, SetLeftExt,
        SetNodeShowHideExt, SetTopExt, SetVisibilityExt, SetWidthExt, StyleBuilder, UiStyleExt,
    },
};

use super::{
    floating_panel::FloatingPanelTitle,
    prelude::{SizedZoneConfig, UiSizedZoneExt, UiTabContainerExt},
    sized_zone::{SizedZone, SizedZonePreUpdate, SizedZoneResizeHandleContainer},
    tab_container::{TabBar, TabContainer, UiTabContainerSubExt},
};

pub struct DockingZonePlugin;

impl Plugin for DockingZonePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, DockingZoneUpdate.after(DroppableUpdate))
            .add_plugins(ComponentThemePlugin::<DockingZoneHighlight>::default())
            .add_systems(
                PreUpdate,
                (
                    cleanup_empty_docking_zones,
                    (
                        cleanup_empty_docking_zone_splits,
                        apply_deferred, // To make sure empties are removed
                        cleanup_shell_docking_zone_splits,
                        apply_deferred, // To make sure double-direction changes are removed
                        cleanup_leftover_docking_zone_splits,
                    )
                        .chain()
                        .run_if(should_cleanup_lingering_docking_zone_splits),
                )
                    .chain()
                    .after(SizedZonePreUpdate),
            )
            .add_systems(
                Update,
                (
                    update_docking_zone_resize_handles.run_if(should_update_resize_handles),
                    handle_docking_zone_drop_zone_change,
                )
                    .in_set(DockingZoneUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct DockingZoneUpdate;

fn cleanup_empty_docking_zones(
    q_tab_containers: Query<(&TabContainer, &RemoveEmptyDockingZone), Changed<TabContainer>>,
    q_parent: Query<&Parent>,
    mut commands: Commands,
) {
    for (tab_container, zone_ref) in &q_tab_containers {
        if tab_container.tab_count() > 0 {
            continue;
        }

        let Ok(parent) = q_parent.get(zone_ref.zone) else {
            warn!(
                "Invalid docking zone detected: Zone {:?} doesn't have a Parent!",
                zone_ref.zone
            );
            commands.entity(zone_ref.zone).despawn_recursive();
            continue;
        };

        let parent_id = parent.get();

        commands.entity(zone_ref.zone).despawn_recursive();
        // Make sure the docking zone's parent is updated to avoid invalid taffy state
        // TODO: remove this once it is no longer needed. Testing should be done by calling despawn_recursive on parent_id in a later frame
        commands.entity(parent_id).add(ResetChildrenInUiSurface);
    }
}

fn should_cleanup_lingering_docking_zone_splits(
    q_zone_splits: Query<Entity, (With<DockingZoneSplitContainer>, Changed<Children>)>,
) -> bool {
    // A split should be removed if:
    // - it has no sized zone child (custom content is removed, it is not supported in DockingZoneSplitContainer)
    //   - spread size proportionally among sized zone siblings
    //   - it's a leaf node and should be just despawn_recursive'd:
    //   - [DockingZoneSplitContainer]
    //     - [SizedZoneResizeHandleContainer]
    //     - [SizedZoneResizeHandleContainer]
    //     - [UnsupportedAdditionalContent]
    // - it is a child of a split and has exactly one sized zone child and no sized zone siblings
    //   - update the child's size to the parent's
    //   - replace parent with the single zone child, depsawn parent and self (refresh new parent's children):
    //   - [DockingZoneSplitContainer:Column]            ->   [SizedZone:Column]
    //       - [DockingZoneSplitContainer:Row]                - [SizedZoneResizeHandleContainer]
    //         - [SizedZone:Column]                           - [SizedZoneResizeHandleContainer]
    //           - [SizedZoneResizeHandleContainer]
    //           - [SizedZoneResizeHandleContainer]
    //         - [SizedZoneResizeHandleContainer]
    //         - [SizedZoneResizeHandleContainer]
    //     - [SizedZoneResizeHandleContainer]
    //     - [SizedZoneResizeHandleContainer]
    //   - if it has multiple sized zone childs, move them to the parent, but spread the size amongst them
    // - any chain of empty splits should be removed
    // This is done in two separate steps, clean up empties first, then clean up nested ones

    q_zone_splits.iter().count() > 0
}

fn cleanup_empty_docking_zone_splits(
    q_zone_splits: Query<(Entity, &Children), (With<DockingZoneSplitContainer>, With<SizedZone>)>,
    q_zone_split: Query<&DockingZoneSplitContainer, With<SizedZone>>,
    q_parent: Query<&Parent>,
    q_children: Query<&Children>,
    mut q_sized_zone: Query<&mut SizedZone>,
    mut commands: Commands,
) {
    // Find zones that have no sized zone children (custom content is removed, it is not supported in DockingZoneSplitContainer)
    //   - spread size equally among sized zone siblings
    //   - it's a leaf node and should be just despawn_recursive'd:
    //   - [DockingZoneSplitContainer]
    //     - [SizedZoneResizeHandleContainer]
    //     - [SizedZoneResizeHandleContainer]
    //     - [UnsupportedAdditionalContent]
    // - any chain of empty splits should be removed
    for (zone_split_id, children) in &q_zone_splits {
        if children
            .iter()
            .any(|child| q_sized_zone.get(*child).is_ok())
        {
            // This is NOT an empty (leaf) split, walk up the tree until one has other branches
            continue;
        }

        let mut topmost_empty_split = zone_split_id;
        let mut topmost_sibling_count = 0;
        for ancestor in q_parent.iter_ancestors(zone_split_id) {
            if q_zone_split.get(ancestor).is_err() {
                // Ancestor isn't a valid docking zone split
                break;
            }

            // Safe unwrap
            let other_branches = q_children.get(ancestor).unwrap();
            let sibling_count = other_branches
                .iter()
                .filter(|branch| {
                    **branch != topmost_empty_split && q_sized_zone.get(**branch).is_ok()
                })
                .count();

            if sibling_count > 0 {
                topmost_sibling_count = sibling_count;
                break;
            }

            topmost_empty_split = ancestor;
        }

        let Ok(parent) = q_parent.get(topmost_empty_split) else {
            warn!(
                "DockingZoneSplitContainer {:?} has no Parent. This is not supported!",
                topmost_empty_split
            );

            commands.entity(topmost_empty_split).despawn_recursive();
            continue;
        };

        let parent_id = parent.get();

        // Distribute removed sized zone size among its siblings
        if topmost_sibling_count > 0 {
            // Safe unwrap, we already checked the topmost is a split
            let sized_zone = q_sized_zone.get(topmost_empty_split).unwrap();
            let split_size = sized_zone.size();
            let Ok(siblings) = q_children.get(parent_id) else {
                unreachable!();
            };

            let sibling_portion = split_size / topmost_sibling_count as f32;
            for sibling in siblings {
                if *sibling == topmost_empty_split {
                    continue;
                }

                let Ok(mut sized_zone) = q_sized_zone.get_mut(*sibling) else {
                    continue;
                };

                let new_size = sized_zone.size() + sibling_portion;
                sized_zone.set_size(new_size);
            }
        }

        commands.entity(topmost_empty_split).despawn_recursive();
        commands.entity(parent_id).add(ResetChildrenInUiSurface);
    }
}

fn cleanup_shell_docking_zone_splits(
    q_zone_splits: Query<
        (Entity, &Children, &Parent),
        (With<DockingZoneSplitContainer>, With<SizedZone>),
    >,
    q_zone_split: Query<&DockingZoneSplitContainer, With<SizedZone>>,
    q_parent: Query<&Parent>,
    q_children: Query<&Children>,
    mut q_sized_zone: Query<&mut SizedZone>,
    mut commands: Commands,
) {
    // Find zone splits that are a child of a split and have more than one sized zone children and no sized zone siblings
    //   - update the child's size to the parent's
    //   - replace parent with the single zone child, depsawn parent and self (refresh new parent's children):
    //   - [DockingZoneSplitContainer:Column]            ->   [SizedZone:Column]
    //       - [DockingZoneSplitContainer:Row]                - [SizedZoneResizeHandleContainer]
    //         - [SizedZone:Column]                           - [SizedZoneResizeHandleContainer]
    //           - [SizedZoneResizeHandleContainer]
    //           - [SizedZoneResizeHandleContainer]
    //         - [SizedZoneResizeHandleContainer]
    //         - [SizedZoneResizeHandleContainer]
    //     - [SizedZoneResizeHandleContainer]
    //     - [SizedZoneResizeHandleContainer]
    //   - if it has multiple sized zone childs, move them to the parent, but spread the size amongst them

    // At this point, there should be no empty docking zone splits, but check anyway
    // Don't traverse the tree upwards to find cleanup opportunities, these should be rare enough to be dealt with in multiple frames if needed
    // Keep track of entities that were moved or deleted, as we don't know the processing order
    let mut entities_to_skip: Vec<Entity> = Vec::with_capacity(q_sized_zone.iter().count());
    for (zone_split_id, children, parent) in &q_zone_splits {
        if entities_to_skip.contains(&zone_split_id) {
            // We already processed this zone
            continue;
        }

        let parent_id = parent.get();
        if entities_to_skip.contains(&parent_id) {
            // We already processed the parent of this zone
            continue;
        }

        if q_zone_split.get(parent_id).is_err() {
            // Parent isn't a docking zone split, keep it.
            continue;
        }

        let Ok(second_parent) = q_parent.get(parent_id) else {
            // The parent split zone doesn't have parent. It is a root node.
            warn!("Docking zone split {:?} doesnt't have a parent!", parent_id);
            continue;
        };
        let second_parent_id = second_parent.get();

        if !children
            .iter()
            .any(|child| q_sized_zone.get(*child).is_ok())
        {
            // Zone split has no sized zone children, it shouldn't be here!
            warn!(
                "Empty docking zone split {:?} detected after cleanup",
                zone_split_id
            );
            continue;
        }

        // Safe unwrap, parent must have at least the current zone
        let siblings = q_children.get(parent_id).unwrap();
        if siblings
            .iter()
            .any(|sibling| *sibling != zone_split_id && q_sized_zone.get(*sibling).is_ok())
        {
            // Split has sized zone siblings, keep the parent
            continue;
        }

        // Don't process ancestors later
        entities_to_skip.push(parent_id);
        entities_to_skip.push(second_parent_id);

        // The parent zone's size needs to be redistributed to the moved children
        // Safe unwarp: already checked that parent is a docking zone split with a sized zone
        let sized_zone = q_sized_zone.get(parent_id).unwrap();
        let split_size_ratio = sized_zone.size() / 100.;
        let mut children_to_move: Vec<Entity> = Vec::with_capacity(children.len());
        for child in children {
            let Ok(mut sized_zone) = q_sized_zone.get_mut(*child) else {
                continue;
            };

            let new_size = sized_zone.size() * split_size_ratio;
            sized_zone.set_size(new_size);

            entities_to_skip.push(*child);
            children_to_move.push(*child);
        }

        // Safe unwraps: a parent found via Parent must have Children, which contains the entity
        let insert_index = q_children
            .get(second_parent_id)
            .unwrap()
            .iter()
            .position(|child| *child == parent_id)
            .unwrap();

        // Move sized zones to outer (second) parent
        commands
            .entity(second_parent_id)
            .insert_children(insert_index, &children_to_move);
        // Remove parent, along with the current split
        commands.entity(parent_id).despawn_recursive();
        // Refresh outer parent's children in taffy to avoid panics
        commands
            .entity(second_parent_id)
            .add(ResetChildrenInUiSurface);
    }
}

fn cleanup_leftover_docking_zone_splits(
    q_zone_splits: Query<
        (Entity, &Children, &Parent),
        (With<DockingZoneSplitContainer>, With<SizedZone>),
    >,
    q_docking_zone: Query<&DockingZone, With<SizedZone>>,
    q_children: Query<&Children>,
    mut q_sized_zone: Query<&mut SizedZone>,
    mut commands: Commands,
) {
    // This is a special case when a DockingZone is the sole remaining child of a docking zone split.
    // The DockingZone cannot have a SizedZone as a direct descedant (not supported), so the direction change can be cleaned up.
    //   - [DockingZoneSplitContainer:Column]            ->   [DockingZone:Column]
    //     - [DockingZone:Row]                                  - [TabContainer]
    //       - [TabContainer]                                   - [SizedZoneResizeHandleContainer]
    //       - [SizedZoneResizeHandleContainer]                 - [SizedZoneResizeHandleContainer]
    //       - [SizedZoneResizeHandleContainer]
    //     - [SizedZoneResizeHandleContainer]
    //     - [SizedZoneResizeHandleContainer]
    // - Make sure to check if there are no other sized zones in the split before moving docking zone up
    for (zone_split_id, zone_split_children, zone_split_parent) in &q_zone_splits {
        if zone_split_children
            .iter()
            .filter(|child| q_sized_zone.get(**child).is_ok())
            .count()
            == 1
            && zone_split_children
                .iter()
                .filter(|child| q_docking_zone.get(**child).is_ok())
                .count()
                == 1
        {
            // Safe unwrap: checked in *if*
            let docking_zone_id = *zone_split_children
                .iter()
                .find(|child| q_docking_zone.get(**child).is_ok())
                .unwrap();

            // Safe unwrap: query is a subset of the query the entity is from
            let split_sized_zone = q_sized_zone.get(zone_split_id).unwrap();
            let size = split_sized_zone.size();

            // Safe unwrap: query is a subset of the query the entity is from
            let mut docking_sized_zone = q_sized_zone.get_mut(docking_zone_id).unwrap();
            docking_sized_zone.set_size(size);

            let zone_split_parent_id = zone_split_parent.get();
            // Safe unwraps: a parent found via Parent must have Children, which contains the entity
            let insert_index = q_children
                .get(zone_split_parent_id)
                .unwrap()
                .iter()
                .position(|child| *child == zone_split_id)
                .unwrap();

            // Move docking zones to split zone parent
            commands
                .entity(zone_split_parent_id)
                .insert_children(insert_index, &vec![docking_zone_id]);
            // Remove split zone
            commands.entity(zone_split_id).despawn_recursive();
            // Refresh parent's children in taffy to avoid panics
            commands
                .entity(zone_split_parent_id)
                .add(ResetChildrenInUiSurface);
        }
    }
}

fn should_update_resize_handles(
    q_accepted_types: Query<&Draggable, (With<FloatingPanelTitle>, Changed<Draggable>)>,
) -> bool {
    q_accepted_types
        .iter()
        .any(|draggable| draggable.state != DragState::Inactive)
}

fn update_docking_zone_resize_handles(
    q_accepted_types: Query<&Draggable, (With<FloatingPanelTitle>, Changed<Draggable>)>,
    q_handle_containers: Query<Entity, With<SizedZoneResizeHandleContainer>>,
    mut commands: Commands,
) {
    let dragging = q_accepted_types.iter().any(|draggable| {
        draggable.state == DragState::DragStart || draggable.state == DragState::Dragging
    });

    for container in &q_handle_containers {
        commands.style(container).render(!dragging);
    }
}

fn handle_docking_zone_drop_zone_change(
    q_docking_zones: Query<
        (Entity, &DockingZone, &DropZone, &Node, &GlobalTransform),
        Changed<DropZone>,
    >,
    q_accepted_query: Query<&FloatingPanelTitle>,
    q_tab_container: Query<&TabContainer>,
    q_tab_bar: Query<(&Node, &Interaction), With<TabBar>>,
    mut commands: Commands,
) {
    for (entity, docking_zone, drop_zone, node, transform) in &q_docking_zones {
        let Ok(tab_container) = q_tab_container.get(docking_zone.tab_container) else {
            warn!("Docking zone {:?} missing its tab container!", entity);
            continue;
        };

        let Ok((tab_bar_node, bar_interaction)) = q_tab_bar.get(tab_container.bar_id()) else {
            warn!(
                "Tab container {:?} missing its tab bar!",
                docking_zone.tab_container
            );
            continue;
        };

        let center = transform.translation().truncate();
        let tab_bar_height = tab_bar_node.size().y;

        if *bar_interaction == Interaction::Hovered
            || drop_zone.drop_phase() == DropPhase::Inactive
            || drop_zone.drop_phase() == DropPhase::DropCanceled
            || drop_zone.drop_phase() == DropPhase::DroppableLeft
            || drop_zone.incoming_droppable().is_none()
            || q_accepted_query
                .get(drop_zone.incoming_droppable().unwrap())
                .is_err()
        {
            commands
                .style(docking_zone.zone_highlight)
                .visibility(Visibility::Hidden);

            continue;
        }

        // How else would the droppable be over the zone?
        let position = drop_zone.position().unwrap();
        let drop_area = calculate_drop_area(position, center, node.size());

        if drop_zone.drop_phase() == DropPhase::DroppableEntered
            || drop_zone.drop_phase() == DropPhase::DroppableHover
        {
            let full_size = Val::Percent(100.);
            let half_size = Val::Percent(50.);
            let auto_size = Val::Auto;

            let (width, height, top, left) = match drop_area {
                DropArea::Center => (
                    full_size,
                    Val::Px(node.size().y - tab_bar_height),
                    Val::Px(tab_bar_height),
                    auto_size,
                ),
                DropArea::North => (full_size, half_size, auto_size, auto_size),
                DropArea::East => (half_size, full_size, auto_size, half_size),
                DropArea::South => (full_size, half_size, half_size, auto_size),
                DropArea::West => (half_size, full_size, auto_size, auto_size),
                _ => (full_size, full_size, auto_size, auto_size),
            };

            commands
                .style(docking_zone.zone_highlight)
                .width(width)
                .height(height)
                .left(left)
                .top(top)
                .visibility(Visibility::Inherited);
        } else if drop_zone.drop_phase() == DropPhase::Dropped {
            // Validated above
            let droppable_title = q_accepted_query
                .get(drop_zone.incoming_droppable().unwrap())
                .unwrap();

            if drop_area == DropArea::Center {
                commands
                    .ui_builder(*tab_container)
                    .dock_panel(droppable_title.panel());
            } else {
                let split_direction = match drop_area {
                    DropArea::North => DockingZoneSplitDirection::VerticallyBefore,
                    DropArea::East => DockingZoneSplitDirection::HorizontallyAfter,
                    DropArea::South => DockingZoneSplitDirection::VerticallyAfter,
                    DropArea::West => DockingZoneSplitDirection::HorizontallyBefore,
                    _ => DockingZoneSplitDirection::VerticallyAfter,
                };

                commands.add(DockingZoneSplit {
                    direction: split_direction,
                    docking_zone: entity,
                    panel_to_dock: droppable_title.panel().into(),
                });
            }

            commands
                .style(docking_zone.zone_highlight)
                .visibility(Visibility::Hidden);
        }
    }
}

fn calculate_drop_area(position: Vec2, center: Vec2, size: Vec2) -> DropArea {
    let sixth_width = size.x / 6.;
    let sixth_height = size.y / 6.;

    if position.x < center.x - sixth_width {
        DropArea::West
    } else if position.x > center.x + sixth_width {
        DropArea::East
    } else if position.y < center.y - sixth_height {
        DropArea::North
    } else if position.y > center.y + sixth_height {
        DropArea::South
    } else {
        DropArea::Center
    }
}

#[derive(PartialEq, Eq)]
enum DockingZoneSplitDirection {
    VerticallyBefore,
    VerticallyAfter,
    HorizontallyBefore,
    HorizontallyAfter,
}

struct DockingZoneSplit {
    docking_zone: Entity,
    direction: DockingZoneSplitDirection,
    panel_to_dock: Option<Entity>,
}

impl Command for DockingZoneSplit {
    fn apply(self, world: &mut World) {
        let Ok((docking_zone, parent, sized_zone)) = world
            .query::<(&DockingZone, &Parent, &SizedZone)>()
            .get(world, self.docking_zone)
        else {
            error!(
                "Tried to split entity {:?} when it isn't a valid DockingZone!",
                self.docking_zone
            );
            return;
        };

        let tab_container_id = docking_zone.tab_container;
        let mut parent_id = parent.get();
        let current_direction = sized_zone.direction();
        let current_size = sized_zone.size();
        let current_min_size = sized_zone.min_size();

        let Some(_) = world.get::<TabContainer>(tab_container_id) else {
            error!(
                "Tab container {:?} missing from docking zone {:?}",
                tab_container_id, self.docking_zone
            );
            return;
        };

        // This must exists, since the Parent exists
        let current_index = world
            .get::<Children>(parent_id)
            .unwrap()
            .iter()
            .position(|child| *child == self.docking_zone)
            .unwrap();

        let (inject_container, sibling_before) = match current_direction {
            FlexDirection::Row => match self.direction {
                DockingZoneSplitDirection::VerticallyBefore => (false, true),
                DockingZoneSplitDirection::VerticallyAfter => (false, false),
                DockingZoneSplitDirection::HorizontallyBefore => (true, true),
                DockingZoneSplitDirection::HorizontallyAfter => (true, false),
            },
            FlexDirection::Column => match self.direction {
                DockingZoneSplitDirection::VerticallyBefore => (true, true),
                DockingZoneSplitDirection::VerticallyAfter => (true, false),
                DockingZoneSplitDirection::HorizontallyBefore => (false, true),
                DockingZoneSplitDirection::HorizontallyAfter => (false, false),
            },
            FlexDirection::RowReverse => match self.direction {
                DockingZoneSplitDirection::VerticallyBefore => (false, false),
                DockingZoneSplitDirection::VerticallyAfter => (false, true),
                DockingZoneSplitDirection::HorizontallyBefore => (true, false),
                DockingZoneSplitDirection::HorizontallyAfter => (true, true),
            },
            FlexDirection::ColumnReverse => match self.direction {
                DockingZoneSplitDirection::VerticallyBefore => (true, false),
                DockingZoneSplitDirection::VerticallyAfter => (true, true),
                DockingZoneSplitDirection::HorizontallyBefore => (false, false),
                DockingZoneSplitDirection::HorizontallyAfter => (false, true),
            },
        };

        // Missing SizedZone on a DockingZone must panic
        let mut sized_zone = world.get_mut::<SizedZone>(self.docking_zone).unwrap();

        let new_container_size = if inject_container {
            50.
        } else {
            current_size / 2.
        };
        sized_zone.set_size(new_container_size);

        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, world);

        if inject_container {
            let new_parent_id = commands
                .ui_builder(parent_id)
                .docking_zone_split(
                    SizedZoneConfig {
                        size: current_size,
                        min_size: current_min_size,
                        ..default()
                    },
                    |_| {},
                )
                .id();

            commands
                .entity(parent_id)
                .insert_children(current_index, &[new_parent_id]);

            parent_id = new_parent_id;
        }

        let new_docking_zone_id = commands
            .ui_builder(parent_id)
            .docking_zone(
                SizedZoneConfig {
                    size: new_container_size,
                    min_size: current_min_size,
                    ..default()
                },
                self.panel_to_dock.is_some(),
                |container| {
                    if let Some(floating_panel_id) = self.panel_to_dock {
                        container.dock_panel(floating_panel_id);
                    }
                },
            )
            .id();

        if inject_container {
            if sibling_before {
                commands.entity(parent_id).add_child(self.docking_zone);
            } else {
                commands
                    .entity(parent_id)
                    .insert_children(0, &[self.docking_zone]);
            }
        } else {
            if sibling_before {
                commands
                    .entity(parent_id)
                    .insert_children(current_index, &[new_docking_zone_id]);
            } else {
                commands
                    .entity(parent_id)
                    .insert_children(current_index + 1, &[new_docking_zone_id]);
            }
        }

        commands.entity(parent_id).reset_children_in_ui_surface();
        queue.apply(world);
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
enum DropArea {
    #[default]
    None,
    Center,
    North,
    East,
    South,
    West,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct DockingZoneSplitContainer;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct DockingZone {
    tab_container: Entity,
    zone_highlight: Entity,
}

impl Default for DockingZone {
    fn default() -> Self {
        Self {
            tab_container: Entity::PLACEHOLDER,
            zone_highlight: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect, UiContext)]
#[reflect(Component)]
pub struct DockingZoneHighlight {
    zone: Entity,
}

impl Default for DockingZoneHighlight {
    fn default() -> Self {
        Self {
            zone: Entity::PLACEHOLDER,
        }
    }
}

impl Default for Theme<DockingZoneHighlight> {
    fn default() -> Self {
        DockingZoneHighlight::theme()
    }
}

impl DockingZoneHighlight {
    pub fn theme() -> Theme<DockingZoneHighlight> {
        let base_theme = PseudoTheme::build(None, DockingZoneHighlight::primary_style);
        let visible_theme = PseudoTheme::deferred(
            vec![PseudoState::Visible],
            DockingZoneHighlight::visible_style,
        );
        Theme::<DockingZoneHighlight>::new(vec![base_theme, visible_theme])
    }

    fn bundle() -> impl Bundle {
        (
            Name::new("Zone Highlight"),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                visibility: Visibility::Hidden,
                ..default()
            },
            LockedStyleAttributes::new(LockableStyleAttribute::FocusPolicy),
            PseudoStates::new(),
        )
    }

    fn primary_style(style_builder: &mut StyleBuilder) {
        style_builder
            .position_type(PositionType::Absolute)
            .width(Val::Percent(100.))
            .height(Val::Percent(100.))
            .background_color(Color::NONE)
            .z_index(ZIndex::Local(100));
    }

    fn visible_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let colors = theme_data.colors();

        style_builder
            .animated()
            .background_color(AnimatedVals {
                idle: colors.accent(Accent::InversePrimary).with_a(0.2),
                enter_from: Some(Color::NONE),
                ..default()
            })
            .copy_from(theme_data.enter_animation);
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct RemoveEmptyDockingZone {
    zone: Entity,
}

impl Default for RemoveEmptyDockingZone {
    fn default() -> Self {
        Self {
            zone: Entity::PLACEHOLDER,
        }
    }
}

pub trait UiDockingZoneExt<'w, 's> {
    fn docking_zone<'a>(
        &'a mut self,
        config: SizedZoneConfig,
        remove_empty: bool,
        spawn_children: impl FnOnce(&mut UiBuilder<TabContainer>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;

    fn docking_zone_split<'a>(
        &'a mut self,
        config: SizedZoneConfig,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiDockingZoneExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn docking_zone<'a>(
        &'a mut self,
        config: SizedZoneConfig,
        remove_empty: bool,
        spawn_children: impl FnOnce(&mut UiBuilder<TabContainer>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut tab_container = Entity::PLACEHOLDER;
        let mut zone_highlight = Entity::PLACEHOLDER;

        let mut docking_zone = self.sized_zone(config, |zone| {
            let zone_id = zone.id();

            let mut new_tab_container = zone.tab_container(spawn_children);
            if remove_empty {
                new_tab_container.insert(RemoveEmptyDockingZone { zone: zone_id });
            }
            tab_container = new_tab_container.id();

            zone_highlight = zone
                .spawn((
                    DockingZoneHighlight::bundle(),
                    DockingZoneHighlight { zone: zone_id },
                ))
                .id();
        });

        docking_zone.insert((
            Name::new("Docking Zone"),
            DockingZone {
                tab_container,
                zone_highlight,
            },
            Interaction::default(),
            DropZone::default(),
            RelativeCursorPosition::default(),
        ));

        docking_zone
    }

    /// Create a sized zone dedicated to holding docking zones. These are the same as what
    /// `DockingZone`s generate when a FloatingPanel is dropped on their sides
    /// NOTE: Custom content will be removed on cleanup.
    fn docking_zone_split<'a>(
        &'a mut self,
        config: SizedZoneConfig,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let new_id = self
            .sized_zone(config, spawn_children)
            .insert((Name::new("Docking Zone Split"), DockingZoneSplitContainer))
            .id();

        self.commands().ui_builder(new_id)
    }
}
