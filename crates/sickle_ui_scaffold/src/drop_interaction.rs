use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::drag_interaction::{DragState, Draggable, DraggableUpdate};

pub struct DropInteractionPlugin;

impl Plugin for DropInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, DroppableUpdate.after(DraggableUpdate))
            .add_systems(
                Update,
                (
                    update_drop_zone_single_frame_state,
                    update_drop_zones.run_if(should_update_drop_zones),
                )
                    .chain()
                    .in_set(DroppableUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct DroppableUpdate;

fn update_drop_zone_single_frame_state(mut q_drop_zones: Query<&mut DropZone, Changed<DropZone>>) {
    for mut drop_zone in &mut q_drop_zones {
        if drop_zone.drop_phase == DropPhase::DroppableLeft
            || drop_zone.drop_phase == DropPhase::DropCanceled
            || drop_zone.drop_phase == DropPhase::Dropped
        {
            drop_zone.drop_phase = DropPhase::Inactive;
            drop_zone.incoming_droppable = None;
            drop_zone.position = None;
        } else if drop_zone.drop_phase == DropPhase::DroppableEntered {
            drop_zone.drop_phase = DropPhase::DroppableHover
        }
    }
}

fn should_update_drop_zones(
    q_droppables: Query<&Draggable, (With<Droppable>, Changed<Draggable>)>,
) -> bool {
    q_droppables.iter().any(|Draggable { state, .. }| {
        *state != DragState::Inactive && *state != DragState::MaybeDragged
    })
}

fn update_drop_zones(
    q_droppables: Query<(Entity, &Draggable), (With<Droppable>, Changed<Draggable>)>,
    q_drop_zone_data: Query<(Entity, &Interaction, &Node, &RelativeCursorPosition), With<DropZone>>,
    mut q_drop_zones: Query<(Entity, &mut DropZone)>,
) {
    // Run condition makes sure we are dragging a droppable.
    // We have no information if the interaction is from the same source,
    // check if cursor is over any.
    //
    // Technically, a pointer / finger could be currently hovering the zone
    // while another pointer / finger drags the droppable, but the ui_focus, flux and
    // drag interactions only track main pointers.
    let mut hovered_to_stack_index: Vec<(Entity, u32)> = q_drop_zone_data
        .iter()
        .filter(|(_, interaction, _, rel_pos)| {
            **interaction == Interaction::Hovered && rel_pos.mouse_over()
        })
        .map(|(entity, _, node, _)| (entity, node.stack_index()))
        .collect();

    hovered_to_stack_index.sort_by_key(|(_, i)| *i);
    if let Some((top_hovered, _)) = hovered_to_stack_index.last() {
        // Safe unwrap: ID comes from a broader query of DropZones.
        let (_, mut drop_zone) = q_drop_zones.get_mut(*top_hovered).unwrap();

        // Take the first droppable that is moving
        // ui_focus, flux and drag interactions only track the main pointer interaction
        let (droppable_entity, draggable) = q_droppables
            .iter()
            .find(|(_, draggable)| {
                draggable.state != DragState::Inactive && draggable.state != DragState::MaybeDragged
            })
            .unwrap();

        // See update_drop_zone_single_frame_state which executes just before this system
        if drop_zone.drop_phase == DropPhase::Inactive {
            drop_zone.drop_phase = DropPhase::DroppableEntered
        } else if draggable.state == DragState::DragEnd {
            drop_zone.drop_phase = DropPhase::Dropped;
        } else if draggable.state == DragState::DragCanceled {
            drop_zone.drop_phase = DropPhase::DropCanceled;
        }

        if draggable.state == DragState::DragStart
            || draggable.state == DragState::Dragging
            || draggable.state == DragState::DragEnd
        {
            drop_zone.incoming_droppable = droppable_entity.into();
            drop_zone.position = draggable.position;
        } else {
            drop_zone.incoming_droppable = None;
            drop_zone.position = None;
        }

        // Update all the other zones
        for (dropzone_id, mut drop_zone) in &mut q_drop_zones {
            if dropzone_id == *top_hovered {
                continue;
            }

            if drop_zone.drop_phase == DropPhase::DroppableEntered
                || drop_zone.drop_phase == DropPhase::DroppableHover
            {
                drop_zone.drop_phase = DropPhase::DroppableLeft;
                drop_zone.incoming_droppable = None;
                drop_zone.position = None;
            }
        }
    } else {
        for (_, mut drop_zone) in &mut q_drop_zones {
            if drop_zone.drop_phase == DropPhase::DroppableEntered
                || drop_zone.drop_phase == DropPhase::DroppableHover
            {
                drop_zone.drop_phase = DropPhase::DroppableLeft;
                drop_zone.incoming_droppable = None;
                drop_zone.position = None;
            }
        }
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
#[reflect]
pub enum DropPhase {
    #[default]
    Inactive,
    DroppableEntered,
    DroppableHover,
    DroppableLeft,
    Dropped,
    DropCanceled,
}

#[derive(Component, Debug, Default, Reflect)]
pub struct Droppable;

#[derive(Component, Debug, Default, Reflect)]
pub struct DropZone {
    drop_phase: DropPhase,
    incoming_droppable: Option<Entity>,
    position: Option<Vec2>,
}

impl DropZone {
    pub fn drop_phase(&self) -> DropPhase {
        self.drop_phase
    }

    pub fn incoming_droppable(&self) -> Option<Entity> {
        self.incoming_droppable
    }

    pub fn position(&self) -> Option<Vec2> {
        self.position
    }
}
