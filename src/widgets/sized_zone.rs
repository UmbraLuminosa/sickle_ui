use bevy::{prelude::*, ui::UiSystem};

use crate::{
    drag_interaction::{DragState, Draggable},
    resize_interaction::{ResizeDirection, ResizeHandle},
    ui_builder::*,
    ui_commands::LogHierarchyExt,
    ui_style::{SetLeftExt, SetTopExt, SetVisibilityExt, UiStyleExt},
};

use super::{docking_zone::DockingZoneUpdate, prelude::UiContainerExt};

const MIN_SIZED_ZONE_SIZE: f32 = 50.;

pub struct SizedZonePlugin;

impl Plugin for SizedZonePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                preset_sized_zone_flex_layout,
                preset_sized_zone_children_size,
                preset_sized_zone_resize_handles,
                preset_sized_zone_border,
            )
                .chain()
                .in_set(SizedZonePreUpdate)
                .run_if(should_update_sized_zone_layout),
        )
        .add_systems(
            Update,
            (update_sized_zone_on_resize, update_sized_zone_style)
                .after(DockingZoneUpdate)
                .chain(),
        )
        .add_systems(
            PostUpdate,
            fit_sized_zones_on_window_resize
                .run_if(should_fit_sized_zones)
                .after(UiSystem::Layout),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct SizedZonePreUpdate;

fn should_update_sized_zone_layout(
    q_added_zones: Query<Entity, Added<SizedZone>>,
    q_parent_changed_zones: Query<Entity, (With<SizedZone>, Changed<Parent>)>,
    q_removed_zones: RemovedComponents<SizedZone>,
) -> bool {
    q_added_zones.iter().count() > 0
        || q_parent_changed_zones.iter().count() > 0
        || q_removed_zones.len() > 0
}

fn preset_sized_zone_flex_layout(
    q_sized_zones: Query<(Entity, &Parent), With<SizedZone>>,
    mut q_sized_zone: Query<&mut SizedZone>,
    q_children: Query<&Children>,
    q_style: Query<&Style>,
) {
    let static_zones: Vec<(Entity, Entity)> = q_sized_zones
        .iter()
        .filter(|(_, parent)| q_sized_zone.get(parent.get()).is_err())
        .map(|(e, p)| (e, p.get()))
        .collect();

    for (sized_zone, parent) in static_zones {
        let Ok(parent_style) = q_style.get(parent) else {
            warn!("No Style found for sized zone parent {:?}!", parent);
            continue;
        };

        let parent_flex_direction = parent_style.flex_direction;
        preset_drop_zone_flex_direction(
            sized_zone,
            &mut q_sized_zone,
            &q_children,
            parent_flex_direction,
        );
    }
}

fn preset_drop_zone_flex_direction(
    sized_zone: Entity,
    q_sized_zone: &mut Query<&mut SizedZone>,
    q_children: &Query<&Children>,
    parent_flex_direction: FlexDirection,
) {
    let mut zone = q_sized_zone.get_mut(sized_zone).unwrap();

    zone.flex_direction = match parent_flex_direction {
        FlexDirection::Row => FlexDirection::Column,
        FlexDirection::Column => FlexDirection::Row,
        FlexDirection::RowReverse => FlexDirection::Column,
        FlexDirection::ColumnReverse => FlexDirection::Row,
    };

    let zone_direction = zone.flex_direction;
    if let Ok(children) = q_children.get(sized_zone) {
        for child in children {
            if q_sized_zone.get(*child).is_ok() {
                preset_drop_zone_flex_direction(*child, q_sized_zone, q_children, zone_direction);
            }
        }
    }
}

fn preset_sized_zone_children_size(
    q_sized_zones: Query<Entity, With<SizedZone>>,
    mut q_sized_zone: Query<&mut SizedZone>,
    q_parents: Query<&Parent>,
) {
    for mut zone in &mut q_sized_zone {
        zone.children_size = 0.;
    }

    for entity in &q_sized_zones {
        let zone = q_sized_zone.get(entity).unwrap();
        let zone_size = zone.min_size;
        let direction = zone.flex_direction;

        for parent in q_parents.iter_ancestors(entity) {
            let Ok(mut parent_zone) = q_sized_zone.get_mut(parent) else {
                continue;
            };

            if parent_zone.flex_direction == direction {
                parent_zone.children_size += zone_size;
            }
        }
    }

    for mut zone in &mut q_sized_zone {
        zone.children_size = zone.children_size.max(zone.min_size);
    }
}

fn preset_sized_zone_border(mut q_sized_zones: Query<(&SizedZone, &mut Style)>) {
    for (zone, mut style) in &mut q_sized_zones {
        match zone.flex_direction {
            FlexDirection::Row => {
                style.border = UiRect::vertical(Val::Px(2.));
            }
            FlexDirection::Column => {
                style.border = UiRect::horizontal(Val::Px(2.));
            }
            _ => (),
        }
    }
}

fn preset_sized_zone_resize_handles(
    q_sized_zone_parents: Query<&Parent, With<SizedZone>>,
    q_children: Query<&Children>,
    q_sized_zones: Query<&SizedZone>,
    q_style: Query<&Style>,
    mut q_resize_handle: Query<&mut SizedZoneResizeHandle>,
    mut commands: Commands,
) {
    let zone_count = q_sized_zone_parents.iter().count();
    let mut handle_visibility: Vec<(Entity, bool)> = Vec::with_capacity(zone_count * 4);
    let mut handle_neighbours: Vec<(Entity, Option<Entity>)> = Vec::with_capacity(zone_count * 4);
    let parents: Vec<Entity> =
        q_sized_zone_parents
            .iter()
            .fold(Vec::with_capacity(zone_count), |mut acc, parent| {
                let entity = parent.get();
                if !acc.contains(&entity) {
                    acc.push(entity);
                }

                acc
            });

    for parent in parents {
        let children: Vec<Entity> = q_children.get(parent).unwrap().iter().map(|e| *e).collect();
        let child_count = children.len();

        if child_count == 1 {
            let Ok(zone) = q_sized_zones.get(children[0]) else {
                return;
            };
            handle_visibility.push((zone.top_handle, false));
            handle_visibility.push((zone.right_handle, false));
            handle_visibility.push((zone.bottom_handle, false));
            handle_visibility.push((zone.left_handle, false));
        } else {
            let mut zone_children: Vec<Entity> = Vec::with_capacity(child_count);
            let mut prev_is_zone = true;

            for i in 0..child_count {
                let Ok(style) = q_style.get(children[i]) else {
                    warn!(
                        "Missing Style detected on Node {:?} during sized zone handle update.",
                        children[i]
                    );
                    commands.entity(children[i]).log_hierarchy(None);
                    continue;
                };

                let Ok(zone) = q_sized_zones.get(children[i]) else {
                    if style.position_type == PositionType::Relative {
                        prev_is_zone = false;
                    }
                    continue;
                };

                match zone.flex_direction {
                    FlexDirection::Row => {
                        handle_visibility.push((zone.top_handle, !prev_is_zone));
                        handle_visibility.push((zone.bottom_handle, i != child_count - 1));
                        handle_visibility.push((zone.right_handle, false));
                        handle_visibility.push((zone.left_handle, false));
                    }
                    FlexDirection::Column => {
                        handle_visibility.push((zone.left_handle, !prev_is_zone));
                        handle_visibility.push((zone.right_handle, i != child_count - 1));
                        handle_visibility.push((zone.top_handle, false));
                        handle_visibility.push((zone.bottom_handle, false));
                    }
                    _ => warn!(
                        "Invalid flex_direction detected on sized zone {:?}",
                        children[i]
                    ),
                }

                prev_is_zone = true;
                zone_children.push(children[i]);
            }

            for i in 0..zone_children.len() {
                let zone = q_sized_zones.get(zone_children[i]).unwrap();
                let Some((prev_handle, next_handle)) = (match zone.flex_direction {
                    FlexDirection::Row => (zone.top_handle, zone.bottom_handle).into(),
                    FlexDirection::Column => (zone.left_handle, zone.right_handle).into(),
                    _ => None,
                }) else {
                    warn!(
                        "Invalid flex_direction detected on sized zone {:?}",
                        zone_children[i]
                    );
                    continue;
                };

                if i == 0 {
                    handle_visibility.push((prev_handle, false));
                }

                if i == zone_children.len() - 1 {
                    handle_visibility.push((next_handle, false));
                }

                handle_neighbours.push((
                    prev_handle,
                    match i > 0 {
                        true => zone_children[i - 1].into(),
                        false => None,
                    },
                ));

                handle_neighbours.push((
                    next_handle,
                    match i < zone_children.len() - 1 {
                        true => zone_children[i + 1].into(),
                        false => None,
                    },
                ));
            }
        }
    }

    for (handle, visible) in handle_visibility {
        commands.style(handle).visibility(match visible {
            true => Visibility::Inherited,
            false => Visibility::Hidden,
        });
    }

    for (handle, neighbour) in handle_neighbours {
        let mut handle = q_resize_handle.get_mut(handle).unwrap();
        handle.neighbour = neighbour;
    }
}

fn update_sized_zone_on_resize(
    q_draggable: Query<(&Draggable, &ResizeHandle, &SizedZoneResizeHandle), Changed<Draggable>>,
    mut q_sized_zone: Query<(&mut SizedZone, &Parent)>,
    q_node: Query<&Node>,
) {
    for (draggable, handle, handle_ref) in &q_draggable {
        if handle_ref.neighbour.is_none() {
            continue;
        }

        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            continue;
        }

        let Some(diff) = draggable.diff else {
            continue;
        };

        let current_zone_id = handle_ref.sized_zone;
        let neighbour_zone_id = handle_ref.neighbour.unwrap();
        let Ok((current_zone, parent)) = q_sized_zone.get(current_zone_id) else {
            continue;
        };
        let Ok((neighbour_zone, other_parent)) = q_sized_zone.get(neighbour_zone_id) else {
            continue;
        };

        if parent != other_parent {
            warn!(
                "Failed to resize sized zone: Neighbouring zones have different parents: {:?} <-> {:?}",
                parent, other_parent
            );
            continue;
        }

        let size_diff = match current_zone.flex_direction {
            FlexDirection::Row => handle.direction().to_size_diff(diff).y,
            FlexDirection::Column => handle.direction().to_size_diff(diff).x,
            _ => 0.,
        };
        if size_diff == 0. {
            continue;
        }

        let Ok(node) = q_node.get(parent.get()) else {
            warn!(
                "Cannot calculate sized zone pixel size: Entity {:?} has parent without Node!",
                current_zone
            );
            continue;
        };

        let total_size = match current_zone.flex_direction {
            FlexDirection::Row => node.size().y,
            FlexDirection::Column => node.size().x,
            _ => 0.,
        };
        if total_size == 0. {
            continue;
        }

        let current_min_size = current_zone.children_size;
        let current_size = (current_zone.size_percent / 100.) * total_size;
        let mut current_new_size = current_size;
        let neighbour_min_size = neighbour_zone.children_size;
        let neighbour_size = (neighbour_zone.size_percent / 100.) * total_size;
        let mut neighbour_new_size = neighbour_size;

        if size_diff < 0. {
            if current_size + size_diff >= current_min_size {
                current_new_size += size_diff;
                neighbour_new_size -= size_diff;
            } else {
                current_new_size = current_min_size;
                neighbour_new_size += current_size - current_min_size;
            }
        } else if size_diff > 0. {
            if neighbour_size - size_diff >= neighbour_min_size {
                neighbour_new_size -= size_diff;
                current_new_size += size_diff;
            } else {
                neighbour_new_size = neighbour_min_size;
                current_new_size += neighbour_size - neighbour_min_size;
            }
        }

        q_sized_zone
            .get_mut(current_zone_id)
            .unwrap()
            .0
            .size_percent = (current_new_size / total_size) * 100.;

        q_sized_zone
            .get_mut(neighbour_zone_id)
            .unwrap()
            .0
            .size_percent = (neighbour_new_size / total_size) * 100.;
    }
}

fn update_sized_zone_style(mut q_sized_zones: Query<(&SizedZone, &mut Style), Changed<SizedZone>>) {
    for (zone, mut style) in &mut q_sized_zones {
        style.flex_direction = zone.flex_direction;
        match zone.flex_direction {
            FlexDirection::Row => {
                style.width = Val::Percent(100.);
                style.height = Val::Percent(zone.size_percent);
            }
            FlexDirection::Column => {
                style.width = Val::Percent(zone.size_percent);
                style.height = Val::Percent(100.);
            }
            _ => (),
        }
    }
}

fn should_fit_sized_zones(
    q_changed_nodes: Query<Entity, (With<SizedZone>, Changed<Node>)>,
    q_removed_zones: RemovedComponents<SizedZone>,
) -> bool {
    q_changed_nodes.iter().count() > 0 || q_removed_zones.len() > 0
}

fn fit_sized_zones_on_window_resize(
    q_children: Query<&Children>,
    q_node: Query<&Node>,
    q_sized_zone_parents: Query<&Parent, With<SizedZone>>,
    q_non_sized: Query<(&Node, &Style), Without<SizedZone>>,
    mut q_sized_zone: Query<(&mut SizedZone, &Node)>,
) {
    let parents: Vec<Entity> = q_sized_zone_parents.iter().fold(
        Vec::with_capacity(q_sized_zone_parents.iter().count()),
        |mut acc, parent| {
            let entity = parent.get();
            if !acc.contains(&entity) {
                acc.push(entity);
            }

            acc
        },
    );

    for parent in parents {
        let Ok(parent_node) = q_node.get(parent) else {
            warn!("Sized zone parent {:?} doesn't have a Node!", parent);
            continue;
        };

        if parent_node.size() == Vec2::ZERO {
            continue;
        }

        let mut non_sized_size = Vec2::ZERO;
        for child in q_children.get(parent).unwrap().iter() {
            if let Ok((node, style)) = q_non_sized.get(*child) {
                if style.position_type == PositionType::Relative {
                    non_sized_size += node.size();
                }
            }
        }

        let mut sum_zone_size = Vec2::ZERO;
        for child in q_children.get(parent).unwrap().iter() {
            if let Ok((_, node)) = q_sized_zone.get(*child) {
                sum_zone_size += node.size();
            };
        }

        for child in q_children.get(parent).unwrap().iter() {
            let Ok((mut sized_zone, zone_node)) = q_sized_zone.get_mut(*child) else {
                continue;
            };

            let total_size = match sized_zone.flex_direction {
                FlexDirection::Row => parent_node.size().y,
                FlexDirection::Column => parent_node.size().x,
                _ => 0.,
            };
            let non_sized_size = match sized_zone.flex_direction {
                FlexDirection::Row => non_sized_size.y,
                FlexDirection::Column => non_sized_size.x,
                _ => 0.,
            };
            let sum_zone_size = match sized_zone.flex_direction {
                FlexDirection::Row => sum_zone_size.y,
                FlexDirection::Column => sum_zone_size.x,
                _ => 0.,
            };

            let sized_size = total_size - non_sized_size;

            if total_size == 0. || sum_zone_size == 0. || sized_size <= 0. {
                continue;
            }

            let multiplier = sized_size / sum_zone_size;
            let own_size = match sized_zone.flex_direction {
                FlexDirection::Row => zone_node.size().y,
                FlexDirection::Column => zone_node.size().x,
                _ => 0.,
            };

            sized_zone.size_percent =
                (own_size.max(sized_zone.children_size) / total_size) * 100. * multiplier;
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SizedZoneResizeHandleContainer;

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct SizedZoneResizeHandle {
    pub sized_zone: Entity,
    pub neighbour: Option<Entity>,
}

impl Default for SizedZoneResizeHandle {
    fn default() -> Self {
        Self {
            sized_zone: Entity::PLACEHOLDER,
            neighbour: Default::default(),
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct SizedZone {
    size_percent: f32,
    min_size: f32,
    children_size: f32,
    flex_direction: FlexDirection,
    top_handle: Entity,
    right_handle: Entity,
    bottom_handle: Entity,
    left_handle: Entity,
}

impl Default for SizedZone {
    fn default() -> Self {
        Self {
            size_percent: Default::default(),
            min_size: MIN_SIZED_ZONE_SIZE,
            children_size: Default::default(),
            flex_direction: Default::default(),
            top_handle: Entity::PLACEHOLDER,
            right_handle: Entity::PLACEHOLDER,
            bottom_handle: Entity::PLACEHOLDER,
            left_handle: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Debug, Default)]
pub struct SizedZoneConfig {
    pub size: f32,
    pub min_size: f32,
}

impl SizedZone {
    pub fn direction(&self) -> FlexDirection {
        self.flex_direction
    }

    pub fn size(&self) -> f32 {
        self.size_percent
    }

    pub fn set_size(&mut self, size: f32) {
        self.size_percent = size.clamp(0., 100.);
    }

    pub fn min_size(&self) -> f32 {
        self.min_size
    }

    fn frame() -> impl Bundle {
        (
            Name::new("Sized Zone"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                border_color: Color::rgb(0.1, 0.1, 0.1).into(),
                ..default()
            },
        )
    }

    fn vertical_handles_container() -> impl Bundle {
        (
            Name::new("Vertical Handles"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
        )
    }
}

pub trait UiSizedZoneExt<'w, 's> {
    fn sized_zone<'a>(
        &'a mut self,
        config: SizedZoneConfig,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiSizedZoneExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn sized_zone<'a>(
        &'a mut self,
        config: SizedZoneConfig,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let size = config.size.clamp(0., 100.);
        let min_size = config.min_size.max(MIN_SIZED_ZONE_SIZE);
        let mut left_handle = Entity::PLACEHOLDER;
        let mut right_handle = Entity::PLACEHOLDER;
        let mut top_handle = Entity::PLACEHOLDER;
        let mut bottom_handle = Entity::PLACEHOLDER;

        let mut sized_zone = self.container(SizedZone::frame(), |container| {
            let zone_id = container.id();
            let handle = SizedZoneResizeHandle {
                sized_zone: zone_id,
                ..default()
            };

            spawn_children(container);

            container.container(
                (
                    ResizeHandle::resize_handle_container(10),
                    SizedZoneResizeHandleContainer,
                ),
                |resize_container| {
                    resize_container.container(
                        (
                            Name::new("Top Row"),
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    height: Val::Px(ResizeHandle::resize_zone_size()),
                                    ..default()
                                },
                                ..default()
                            },
                        ),
                        |top_row| {
                            top_handle = top_row
                                .spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::North),
                                    handle,
                                ))
                                .style()
                                .left(Val::Px(0.))
                                .id();
                        },
                    );

                    resize_container.container(
                        (
                            Name::new("Bottom Row"),
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    height: Val::Px(ResizeHandle::resize_zone_size()),
                                    ..default()
                                },
                                ..default()
                            },
                        ),
                        |bottom_row| {
                            bottom_handle = bottom_row
                                .spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::South),
                                    handle,
                                ))
                                .style()
                                .left(Val::Px(0.))
                                .id();
                        },
                    );
                },
            );

            container.container(
                (
                    ResizeHandle::resize_handle_container(11),
                    SizedZoneResizeHandleContainer,
                ),
                |resize_container| {
                    resize_container.container(
                        SizedZone::vertical_handles_container(),
                        |middle_row| {
                            left_handle = middle_row
                                .spawn((ResizeHandle::resize_handle(ResizeDirection::West), handle))
                                .style()
                                .top(Val::Px(0.))
                                .id();
                            right_handle = middle_row
                                .spawn((ResizeHandle::resize_handle(ResizeDirection::East), handle))
                                .style()
                                .top(Val::Px(0.))
                                .id();
                        },
                    );
                },
            );
        });

        sized_zone.insert(SizedZone {
            size_percent: size,
            min_size,
            top_handle,
            right_handle,
            bottom_handle,
            left_handle,
            ..default()
        });

        sized_zone
    }
}
