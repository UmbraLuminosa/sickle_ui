use std::{cmp::Ordering, ops::Add, time::Duration};

use bevy::{prelude::*, time::Stopwatch, utils::HashMap};
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

impl FluxInteraction {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
    pub fn is_pointer_enter(&self) -> bool {
        matches!(self, Self::PointerEnter)
    }
    pub fn is_pointer_leave(&self) -> bool {
        matches!(self, Self::PointerLeave)
    }
    pub fn is_pressed(&self) -> bool {
        matches!(self, Self::Pressed)
    }
    pub fn is_released(&self) -> bool {
        matches!(self, Self::Released)
    }
    pub fn is_canceled(&self) -> bool {
        matches!(self, Self::PressCanceled)
    }
    pub fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }
}

#[derive(Component, Clone, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct FluxInteractionStopwatch(pub Stopwatch);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StopwatchLock {
    #[default]
    None,
    Infinite,
    Duration(Duration),
}

impl Add for StopwatchLock {
    type Output = StopwatchLock;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (StopwatchLock::None, StopwatchLock::None) => StopwatchLock::None,
            (StopwatchLock::None, StopwatchLock::Duration(_)) => rhs,
            (StopwatchLock::Duration(_), StopwatchLock::None) => self,
            (StopwatchLock::Duration(l_duration), StopwatchLock::Duration(r_duration)) => {
                StopwatchLock::Duration(l_duration + r_duration)
            }
            // Either side is infinite, let them cook
            _ => StopwatchLock::Infinite,
        }
    }
}

impl Ord for StopwatchLock {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (StopwatchLock::None, StopwatchLock::None) => Ordering::Equal,
            (StopwatchLock::None, StopwatchLock::Infinite) => Ordering::Less,
            (StopwatchLock::None, StopwatchLock::Duration(_)) => Ordering::Less,
            (StopwatchLock::Infinite, StopwatchLock::None) => Ordering::Greater,
            (StopwatchLock::Infinite, StopwatchLock::Infinite) => Ordering::Equal,
            (StopwatchLock::Infinite, StopwatchLock::Duration(_)) => Ordering::Greater,
            (StopwatchLock::Duration(_), StopwatchLock::None) => Ordering::Greater,
            (StopwatchLock::Duration(_), StopwatchLock::Infinite) => Ordering::Less,
            (StopwatchLock::Duration(lv), StopwatchLock::Duration(rv)) => lv.cmp(rv),
        }
    }
}

impl PartialOrd for StopwatchLock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (StopwatchLock::None, StopwatchLock::None) => Some(Ordering::Equal),
            (StopwatchLock::None, StopwatchLock::Infinite) => Some(Ordering::Less),
            (StopwatchLock::None, StopwatchLock::Duration(_)) => Some(Ordering::Less),
            (StopwatchLock::Infinite, StopwatchLock::None) => Some(Ordering::Greater),
            (StopwatchLock::Infinite, StopwatchLock::Infinite) => Some(Ordering::Equal),
            (StopwatchLock::Infinite, StopwatchLock::Duration(_)) => Some(Ordering::Greater),
            (StopwatchLock::Duration(_), StopwatchLock::None) => Some(Ordering::Greater),
            (StopwatchLock::Duration(_), StopwatchLock::Infinite) => Some(Ordering::Less),
            (StopwatchLock::Duration(lv), StopwatchLock::Duration(rv)) => lv.partial_cmp(rv),
        }
    }
}

#[derive(Component, Clone, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct FluxInteractionStopwatchLock(HashMap<&'static str, StopwatchLock>);

impl FluxInteractionStopwatchLock {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn min_duration(&self) -> StopwatchLock {
        if self.is_empty() {
            return StopwatchLock::None;
        }

        let mut values: Vec<StopwatchLock> = self.0.values().into_iter().copied().collect();
        values.sort();

        // Safe unwrap: empty checked above
        *(values.last().unwrap())
    }

    pub fn lock(&mut self, owner: &'static str, duration: StopwatchLock) {
        self.0.insert(owner, duration);
    }

    pub fn release(&mut self, lock_of: &'static str) {
        self.0.remove(lock_of);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
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
    mut q_stopwatch: Query<(
        Entity,
        &mut FluxInteractionStopwatch,
        Option<&FluxInteractionStopwatchLock>,
    )>,
    mut commands: Commands,
) {
    for (entity, mut stopwatch, lock) in &mut q_stopwatch {
        let remove_stopwatch = if let Some(lock) = lock {
            match lock.min_duration() {
                StopwatchLock::None => {
                    stopwatch.0.elapsed().as_secs_f32() > config.max_interaction_duration
                }
                StopwatchLock::Infinite => false,
                StopwatchLock::Duration(length) => stopwatch.0.elapsed() > length,
            }
        } else {
            stopwatch.0.elapsed().as_secs_f32() > config.max_interaction_duration
        };

        if remove_stopwatch {
            commands.entity(entity).remove::<FluxInteractionStopwatch>();
        }

        stopwatch.0.tick(time.delta());
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
