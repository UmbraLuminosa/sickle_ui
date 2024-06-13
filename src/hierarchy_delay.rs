use bevy::{ecs::system::EntityCommands, prelude::*, ui::UiSystem};

use sickle_ui_scaffold::ui_commands::ResetChildrenInUiSurface;

pub struct HierarchyDelayPlugin;

impl Plugin for HierarchyDelayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            update_ui_surface
                .after(UiSystem::Layout)
                .in_set(HierarchyDelayPreUpdate),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct HierarchyDelayPreUpdate;

fn update_ui_surface(
    q_parents: Query<Entity, (With<DelayResetChildrenInUiSurface>, With<Children>)>,
    mut commands: Commands,
) {
    for parent in &q_parents {
        commands
            .entity(parent)
            .add(ResetChildrenInUiSurface)
            .remove::<DelayResetChildrenInUiSurface>();
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct DelayResetChildrenInUiSurface;

pub trait DelayActions {
    fn reset_children_in_ui_surface(&mut self) -> &mut Self;
}

impl DelayActions for EntityCommands<'_> {
    fn reset_children_in_ui_surface(&mut self) -> &mut Self {
        self.insert(DelayResetChildrenInUiSurface);

        self
    }
}
