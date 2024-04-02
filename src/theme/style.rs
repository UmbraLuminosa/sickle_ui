use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;
use sickle_macros::StyleCommands;

use crate::{ui_style::UiStyle, FluxInteraction};

// TODO: Add support for continous animations, i.e. loop, ping-pong
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AnimationProgress {
    #[default]
    Start,
    Inbetween(f32),
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InteractionAnimationState {
    phase: crate::FluxInteraction,
    iteration: u8,
    animation: AnimationProgress,
}

pub struct StaticValueBundle<T: Default + Clone + Copy + PartialEq> {
    base: T,
    hover: Option<T>,
    press: Option<T>,
    cancel: Option<T>,
    focus: Option<T>,
}

impl<T: Default + Clone + Copy + PartialEq> StaticValueBundle<T> {
    fn to_value(&self, flux_interaction: crate::FluxInteraction) -> T {
        self.base
    }
}

pub struct AnimatedValueBundle<T: sickle_math::lerp::Lerp + Default + Clone + Copy + PartialEq> {
    base: T,
    hover: Option<T>,
    press: Option<T>,
    cancel: Option<T>,
    focus: Option<T>,
}

impl<T: sickle_math::lerp::Lerp + Default + Clone + Copy + PartialEq> AnimatedValueBundle<T> {
    fn to_value(
        &self,
        transition_base: InteractionAnimationState,
        animation_progress: InteractionAnimationState,
    ) -> T {
        // TODO: LERP
        self.base
    }
}

pub struct CustomInteractiveStyleAttribute {
    callback: fn(Entity, FluxInteraction, &mut World),
    flux_interaction: FluxInteraction,
}

impl EntityCommand for CustomInteractiveStyleAttribute {
    fn apply(self, id: Entity, world: &mut World) {
        (self.callback)(id, self.flux_interaction, world);
    }
}

pub struct CustomAnimatableStyleAttribute {
    callback: fn(Entity, InteractionAnimationState, InteractionAnimationState, &mut World),
    transition_base: InteractionAnimationState,
    animation_progress: InteractionAnimationState,
}

impl EntityCommand for CustomAnimatableStyleAttribute {
    fn apply(self, id: Entity, world: &mut World) {
        (self.callback)(id, self.transition_base, self.animation_progress, world);
    }
}

// pub enum StaticStyleAttribute {
//     BackgroundColor(Color),
//     Custom(fn(Entity, &mut World)),
// }

// impl PartialEq for StaticStyleAttribute {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (Self::BackgroundColor(_), Self::BackgroundColor(_)) => true,
//             (Self::Custom(l0), Self::Custom(r0)) => l0 == r0,
//             _ => false,
//         }
//     }
// }

// impl StaticStyleAttribute {
//     pub fn apply(&self, ui_style: &mut UiStyle) {
//         match self {
//             Self::BackgroundColor(value) => todo!(), //ui_style.background_color(value),
//             Self::Custom(callback) => {
//                 ui_style.entity_commands().add(*callback);
//             }
//         }
//     }
// }

// equality only on base value
pub enum InteractiveStyleAttribute {
    BackgroundColor(StaticValueBundle<Color>),
    Custom(fn(Entity, FluxInteraction, &mut World)),
}

impl PartialEq for InteractiveStyleAttribute {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BackgroundColor(_), Self::BackgroundColor(_)) => true,
            (Self::Custom(l0), Self::Custom(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl InteractiveStyleAttribute {
    fn to_attribute(&self, flux_interaction: FluxInteraction) -> StaticStyleAttribute {
        match self {
            Self::BackgroundColor(bundle) => {
                StaticStyleAttribute::BackgroundColor(bundle.to_value(flux_interaction))
            }
            Self::Custom(_) => StaticStyleAttribute::Custom(|_, _| {
                error!("Custom InteractiveStyleAttribute marshalled to StaticStyleAttribute!");
            }),
        }
    }

    pub fn apply(&self, flux_interaction: FluxInteraction, ui_style: &mut UiStyle) {
        match self {
            Self::Custom(callback) => {
                ui_style
                    .entity_commands()
                    .add(CustomInteractiveStyleAttribute {
                        callback: *callback,
                        flux_interaction,
                    });
            }
            _ => self.to_attribute(flux_interaction).apply(ui_style),
        }
    }
}

pub enum AnimatedStyleAttribute {
    BackgroundColor(AnimatedValueBundle<Color>),
    Custom(fn(Entity, InteractionAnimationState, InteractionAnimationState, &mut World)),
}

impl PartialEq for AnimatedStyleAttribute {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BackgroundColor(_), Self::BackgroundColor(_)) => true,
            (Self::Custom(l0), Self::Custom(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl AnimatedStyleAttribute {
    fn to_attribute(
        &self,
        transition_base: InteractionAnimationState,
        animation_progress: InteractionAnimationState,
    ) -> StaticStyleAttribute {
        match self {
            Self::BackgroundColor(bundle) => StaticStyleAttribute::BackgroundColor(
                bundle.to_value(transition_base, animation_progress),
            ),
            Self::Custom(_) => StaticStyleAttribute::Custom(|_, _| {
                error!("Custom AnimatedStyleAttribute marshalled to StaticStyleAttribute!");
            }),
        }
    }

    pub fn apply(
        &self,
        transition_base: InteractionAnimationState,
        animation_progress: InteractionAnimationState,
        ui_style: &mut UiStyle,
    ) {
        match self {
            Self::Custom(callback) => {
                ui_style
                    .entity_commands()
                    .add(CustomAnimatableStyleAttribute {
                        callback: *callback,
                        transition_base,
                        animation_progress,
                    });
            }
            _ => self
                .to_attribute(transition_base, animation_progress)
                .apply(ui_style),
        }
    }
}

#[derive(StyleCommands)]
enum _StyleAttributes {
    // Display { display: bevy::ui::Display },
    // PositionType {
    //     position_type: PositionType,
    // },
    // Overflow {
    //     overflow: Overflow,
    // },
    // Direction {
    //     direction: Direction,
    // },
    // #[animatable]
    // Left {
    //     left: bevy::ui::Val,
    // },
    #[target_tupl(BackgroundColor)]
    #[animatable]
    BackgroundColor {
        background_color: bevy::render::color::Color,
    },
    // #[target_tupl(BorderColor)]
    // BorderColor {
    //     border_color: bevy::render::color::Color,
    // },
    // #[target_enum]
    // FocusPolicy {
    //     focus_policy: bevy::ui::FocusPolicy,
    // },
    // #[target_enum]
    // Visibility {
    //     visibility: bevy::render::view::Visibility,
    // },
    // #[skip_enity_command]
    // ZIndex {
    //     z_index: bevy::ui::ZIndex,
    // },
    // #[skip_ui_style_ext]
    // Image {
    //     image: String,
    // },
}
