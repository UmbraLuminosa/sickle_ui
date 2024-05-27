use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

pub struct ScrollInteractionPlugin;

impl Plugin for ScrollInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, ScrollableUpdate)
            .add_systems(Update, update_scrollables.in_set(ScrollableUpdate));
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct ScrollableUpdate;

fn update_scrollables(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    r_keys: Res<ButtonInput<KeyCode>>,
    mut q_scrollables: Query<(&mut Scrollable, &Interaction)>,
) {
    let mut axis = ScrollAxis::Vertical;
    let mut offset = 0.;
    let mut unit = MouseScrollUnit::Line;
    let mut has_event = false;

    // Only the last event is kept
    for mouse_wheel_event in mouse_wheel_events.read() {
        axis = ScrollAxis::Vertical;
        unit = mouse_wheel_event.unit;
        offset = if mouse_wheel_event.x != 0. {
            -mouse_wheel_event.x
        } else {
            -mouse_wheel_event.y
        };

        if mouse_wheel_event.x > 0. || r_keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        {
            axis = ScrollAxis::Horizontal;
        }

        has_event = true;
    }

    if !has_event {
        return;
    }

    for (mut scrollable, interaction) in &mut q_scrollables {
        if *interaction != Interaction::Hovered {
            continue;
        }

        scrollable.axis = axis.into();
        scrollable.diff = offset.into();
        scrollable.unit = unit;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
pub enum ScrollAxis {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Scrollable {
    axis: Option<ScrollAxis>,
    diff: f32,
    unit: MouseScrollUnit,
}

impl Default for Scrollable {
    fn default() -> Self {
        Self {
            axis: Default::default(),
            diff: Default::default(),
            unit: MouseScrollUnit::Pixel,
        }
    }
}

impl Scrollable {
    pub fn last_change(&self) -> Option<(ScrollAxis, f32, MouseScrollUnit)> {
        let Some(axis) = self.axis else {
            return None;
        };

        (axis, self.diff, self.unit).into()
    }
}
