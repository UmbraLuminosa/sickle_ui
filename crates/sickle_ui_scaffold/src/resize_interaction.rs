use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    drag_interaction::Draggable,
    interactions::InteractiveBackground,
    ui_commands::SetCursorExt,
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

pub struct ResizeHandlePlugin;

impl Plugin for ResizeHandlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_cursor_on_resize_handles
                .run_if(should_update_resize_handle_cursor)
                .after(FluxInteractionUpdate),
        );
    }
}

fn should_update_resize_handle_cursor(
    q_flux: Query<&ResizeHandle, Changed<FluxInteraction>>,
) -> bool {
    q_flux.iter().count() > 0
}

fn update_cursor_on_resize_handles(
    q_flux: Query<(&ResizeHandle, &FluxInteraction)>,
    mut locked: Local<bool>,
    mut commands: Commands,
) {
    let mut new_cursor: Option<CursorIcon> = None;
    let multiple_active = q_flux
        .iter()
        .filter(|(_, flux)| {
            (**flux == FluxInteraction::PointerEnter && !*locked)
                || **flux == FluxInteraction::Pressed
        })
        .count()
        > 1;

    let omni_cursor = CursorIcon::Move;

    for (handle, flux) in &q_flux {
        match *flux {
            FluxInteraction::PointerEnter => {
                if !*locked {
                    new_cursor = match multiple_active {
                        true => omni_cursor.into(),
                        false => handle.direction.cursor().into(),
                    };
                }
            }
            FluxInteraction::Pressed => {
                new_cursor = match multiple_active {
                    true => omni_cursor.into(),
                    false => handle.direction.cursor().into(),
                };
                *locked = true;
            }
            FluxInteraction::Released => {
                *locked = false;
                if new_cursor.is_none() {
                    new_cursor = CursorIcon::Default.into();
                }
            }
            FluxInteraction::PressCanceled => {
                *locked = false;
                if new_cursor.is_none() {
                    new_cursor = CursorIcon::Default.into();
                }
            }
            FluxInteraction::PointerLeave => {
                if !*locked && new_cursor.is_none() {
                    new_cursor = CursorIcon::Default.into();
                }
            }
            _ => (),
        }
    }

    if let Some(new_cursor) = new_cursor {
        commands.set_cursor(new_cursor);
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ResizeHandle {
    direction: ResizeDirection,
}

impl ResizeHandle {
    pub fn direction(&self) -> ResizeDirection {
        self.direction
    }

    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    pub fn resize_zone_size() -> f32 {
        4.
    }

    pub fn resize_zone_pullback() -> f32 {
        2.
    }

    pub fn resize_handle_container(elevation: i32) -> impl Bundle {
        (
            Name::new("Resize Handle Container"),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::SpaceBetween,
                    align_self: AlignSelf::Stretch,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                z_index: ZIndex::Local(elevation),
                ..default()
            },
        )
    }

    pub fn resize_handle(direction: ResizeDirection) -> impl Bundle {
        let zone_size = ResizeHandle::resize_zone_size();

        let (width, height) = match direction {
            ResizeDirection::North => (Val::Percent(100.), Val::Px(zone_size)),
            ResizeDirection::NorthEast => (Val::Px(zone_size), Val::Px(zone_size)),
            ResizeDirection::East => (Val::Px(zone_size), Val::Percent(100.)),
            ResizeDirection::SouthEast => (Val::Px(zone_size), Val::Px(zone_size)),
            ResizeDirection::South => (Val::Percent(100.), Val::Px(zone_size)),
            ResizeDirection::SouthWest => (Val::Px(zone_size), Val::Px(zone_size)),
            ResizeDirection::West => (Val::Px(zone_size), Val::Percent(100.)),
            ResizeDirection::NorthWest => (Val::Px(zone_size), Val::Px(zone_size)),
        };
        let name = match direction {
            ResizeDirection::North => "North",
            ResizeDirection::NorthEast => "NorthEast",
            ResizeDirection::East => "East",
            ResizeDirection::SouthEast => "SouthEast",
            ResizeDirection::South => "South",
            ResizeDirection::SouthWest => "SouthWest",
            ResizeDirection::West => "West",
            ResizeDirection::NorthWest => "NorthWest",
        };

        let pullback = Val::Px(-ResizeHandle::resize_zone_pullback());
        (
            Name::new(format!("Resize Handle: [{}]", name)),
            NodeBundle {
                style: Style {
                    top: pullback,
                    left: pullback,
                    width,
                    height,
                    ..default()
                },
                focus_policy: bevy::ui::FocusPolicy::Pass,
                ..default()
            },
            Interaction::default(),
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Color::rgb(0., 0.5, 1.).into(),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: ResizeHandle::base_tween(),
                ..default()
            },
            Draggable::default(),
            RelativeCursorPosition::default(),
            ResizeHandle { direction },
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
pub enum ResizeDirection {
    #[default]
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl ResizeDirection {
    pub fn cursor(&self) -> CursorIcon {
        match self {
            ResizeDirection::North => CursorIcon::NResize,
            ResizeDirection::NorthEast => CursorIcon::NeResize,
            ResizeDirection::East => CursorIcon::EResize,
            ResizeDirection::SouthEast => CursorIcon::SeResize,
            ResizeDirection::South => CursorIcon::SResize,
            ResizeDirection::SouthWest => CursorIcon::SwResize,
            ResizeDirection::West => CursorIcon::WResize,
            ResizeDirection::NorthWest => CursorIcon::NwResize,
        }
    }

    pub fn to_size_diff(&self, drag_diff: Vec2) -> Vec2 {
        match self {
            ResizeDirection::North => Vec2 {
                x: 0.,
                y: -drag_diff.y,
            },
            ResizeDirection::NorthEast => Vec2 {
                x: drag_diff.x,
                y: -drag_diff.y,
            },
            ResizeDirection::East => Vec2 {
                x: drag_diff.x,
                y: 0.,
            },
            ResizeDirection::SouthEast => drag_diff,
            ResizeDirection::South => Vec2 {
                x: 0.,
                y: drag_diff.y,
            },
            ResizeDirection::SouthWest => Vec2 {
                x: -drag_diff.x,
                y: drag_diff.y,
            },
            ResizeDirection::West => Vec2 {
                x: -drag_diff.x,
                y: 0.,
            },
            ResizeDirection::NorthWest => Vec2 {
                x: -drag_diff.x,
                y: -drag_diff.y,
            },
        }
    }
}
