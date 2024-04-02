use bevy::{prelude::*, time::Stopwatch};
use bevy_reflect::Reflect;

pub struct FluxInteractionPlugin;

impl Plugin for FluxInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FluxInteractionConfig>()
            .configure_sets(Update, FluxInteractionUpdate)
            .add_systems(
                Update,
                (
                    tick_flux_interaction_stopwatch,
                    update_flux_interaction,
                    reset_stopwatch_on_change,
                    update_prev_interaction,
                )
                    .chain()
                    .in_set(FluxInteractionUpdate),
            );
    }
}

// TODO: calculate value based on theme tween lengths and submenu timings
#[derive(Resource, Clone, Debug, Reflect)]
pub struct FluxInteractionConfig {
    pub max_interaction_duration: f32,
}

impl Default for FluxInteractionConfig {
    fn default() -> Self {
        Self {
            max_interaction_duration: 1.,
        }
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct FluxInteractionUpdate;

#[derive(Bundle, Clone, Debug, Default)]
pub struct TrackedInteraction {
    pub interaction: FluxInteraction,
    pub prev_interaction: PrevInteraction,
    pub stopwatch: FluxInteractionStopwatch,
}

#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
#[reflect(Component, PartialEq)]
pub enum FluxInteraction {
    #[default]
    None,
    PointerEnter,
    PointerLeave,
    /// Pressing started, but not completed or cancelled
    Pressed,
    /// Pressing completed over the node
    Released,
    /// Pressing cancelled by releasing outside of node
    PressCanceled,
    Disabled,
}

#[derive(Component, Clone, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct FluxInteractionStopwatch(pub Stopwatch);

// TODO: Add FluxInteractionStopwatchLock (sparse set)

#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
#[reflect(Component, PartialEq)]
pub enum PrevInteraction {
    #[default]
    None,
    Pressed,
    Hovered,
}

fn tick_flux_interaction_stopwatch(
    config: Res<FluxInteractionConfig>,
    time: Res<Time<Real>>,
    mut q_stopwatch: Query<(Entity, &mut FluxInteractionStopwatch)>,
    mut commands: Commands,
) {
    for (entity, mut stopwatch) in &mut q_stopwatch {
        if stopwatch.0.elapsed().as_secs_f32() > config.max_interaction_duration {
            commands.entity(entity).remove::<FluxInteractionStopwatch>();
        } else {
            stopwatch.0.tick(time.delta());
        }
    }
}

fn update_flux_interaction(
    mut q_interaction: Query<
        (&PrevInteraction, &Interaction, &mut FluxInteraction),
        Changed<Interaction>,
    >,
) {
    for (prev, curr, mut flux) in &mut q_interaction {
        if *flux == FluxInteraction::Disabled {
            continue;
        }

        if *prev == PrevInteraction::None && *curr == Interaction::Hovered {
            *flux = FluxInteraction::PointerEnter;
        } else if *prev == PrevInteraction::None && *curr == Interaction::Pressed
            || *prev == PrevInteraction::Hovered && *curr == Interaction::Pressed
        {
            *flux = FluxInteraction::Pressed;
        } else if *prev == PrevInteraction::Hovered && *curr == Interaction::None {
            *flux = FluxInteraction::PointerLeave;
        } else if *prev == PrevInteraction::Pressed && *curr == Interaction::None {
            *flux = FluxInteraction::PressCanceled;
        } else if *prev == PrevInteraction::Pressed && *curr == Interaction::Hovered {
            *flux = FluxInteraction::Released;
        }
    }
}

fn reset_stopwatch_on_change(
    mut q_stopwatch: Query<
        (Entity, Option<&mut FluxInteractionStopwatch>),
        Changed<FluxInteraction>,
    >,
    mut commands: Commands,
) {
    for (entity, stopwatch) in &mut q_stopwatch {
        if let Some(mut stopwatch) = stopwatch {
            stopwatch.0.reset();
        } else {
            commands
                .entity(entity)
                .insert(FluxInteractionStopwatch::default());
        }
    }
}

fn update_prev_interaction(
    mut q_interaction: Query<(&mut PrevInteraction, &Interaction), Changed<Interaction>>,
) {
    for (mut prev_interaction, interaction) in &mut q_interaction {
        *prev_interaction = match *interaction {
            Interaction::Pressed => PrevInteraction::Pressed,
            Interaction::Hovered => PrevInteraction::Hovered,
            Interaction::None => PrevInteraction::None,
        };
    }
}
