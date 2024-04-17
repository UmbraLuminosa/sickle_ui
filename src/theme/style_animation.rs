use std::{num::NonZeroU8, time::Duration, vec};

use bevy::prelude::*;
use sickle_math::{
    ease::{Ease, ValueEasing},
    lerp::Lerp,
};

use crate::{ui_style::AnimatedBundle, FluxInteraction, FluxInteractionStopwatch, StopwatchLock};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InteractionStyle {
    #[default]
    Base,
    Hover,
    Press,
    Cancel,
}

impl From<FluxInteraction> for InteractionStyle {
    fn from(value: FluxInteraction) -> Self {
        match value {
            FluxInteraction::None => Self::Base,
            FluxInteraction::PointerEnter => Self::Hover,
            FluxInteraction::PointerLeave => Self::Base,
            FluxInteraction::Pressed => Self::Press,
            FluxInteraction::Released => Self::Hover,
            FluxInteraction::PressCanceled => Self::Cancel,
            FluxInteraction::Disabled => Self::Base,
        }
    }
}

impl From<&FluxInteraction> for InteractionStyle {
    fn from(value: &FluxInteraction) -> Self {
        Self::from(*value)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationState {
    result: AnimationResult,
    iteration: u8,
}

impl AnimationState {
    pub fn extract<T: Lerp + Default + Clone + Copy + PartialEq>(
        &self,
        bundle: &AnimatedBundle<T>,
    ) -> T {
        self.result.extract(bundle)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationResult {
    Hold(InteractionStyle),
    Interpolate(InteractionStyle, InteractionStyle, f32),
    Reverse(InteractionStyle, InteractionStyle, f32, f32),
    TransitionBetween {
        origin: InteractionStyle,
        points: Vec<(InteractionStyle, f32)>,
    },
}

impl Default for AnimationResult {
    fn default() -> Self {
        Self::Hold(Default::default())
    }
}

impl AnimationResult {
    pub fn extract<T: Lerp + Default + Clone + Copy + PartialEq>(
        &self,
        bundle: &AnimatedBundle<T>,
    ) -> T {
        match self {
            AnimationResult::Hold(style) => bundle.interaction_style(*style),
            AnimationResult::Interpolate(start_style, end_style, t) => bundle
                .interaction_style(*start_style)
                .lerp(bundle.interaction_style(*end_style), *t),
            AnimationResult::Reverse(start_style, end_style, t, _) => bundle
                .interaction_style(*start_style)
                .lerp(bundle.interaction_style(*end_style), *t),
            AnimationResult::TransitionBetween { origin, points } => {
                let start_value = bundle.interaction_style(*origin);
                points
                    .iter()
                    .fold(start_value, |current_value, (style, t)| {
                        current_value.lerp(bundle.interaction_style(*style), *t)
                    })
            }
        }
    }
}

//  TODO: Add support for continous animations, i.e. loop, ping-pong
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AnimationLoop {
    #[default]
    None,
    Continous,
    Times(NonZeroU8),
    PingPongContinous,
    PingPong(NonZeroU8),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AnimationConfig {
    duration: f32,
    easing: Option<Ease>,
    delay: Option<f32>,
    loop_type: Option<AnimationLoop>,
}

impl AnimationConfig {
    fn delay(&self) -> f32 {
        match self.delay {
            Some(delay) => delay,
            None => 0.,
        }
    }

    fn easing(&self) -> Ease {
        match self.easing {
            Some(ease) => ease,
            None => Ease::Linear,
        }
    }

    fn loop_type(&self) -> AnimationLoop {
        match self.loop_type {
            Some(loop_type) => loop_type,
            None => AnimationLoop::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StyleAnimation {
    pointer_enter: Option<AnimationConfig>,
    pointer_leave: Option<AnimationConfig>,
    press: Option<AnimationConfig>,
    release: Option<AnimationConfig>,
    cancel: Option<AnimationConfig>,
    cancel_reset: Option<AnimationConfig>,
}

macro_rules! animation_setter {
    ($setter:ident) => {
        pub fn $setter(
            &mut self,
            duration: f32,
            easing: impl Into<Option<Ease>>,
            delay: impl Into<Option<f32>>,
            loop_type: impl Into<Option<AnimationLoop>>,
        ) -> &mut StyleAnimation {
            let config = AnimationConfig {
                duration,
                easing: easing.into(),
                delay: delay.into(),
                loop_type: loop_type.into(),
            };
            self.$setter = Some(config);

            self
        }
    };
}

impl StyleAnimation {
    pub fn new() -> Self {
        Self { ..default() }
    }

    animation_setter!(pointer_enter);
    animation_setter!(pointer_leave);
    animation_setter!(press);
    animation_setter!(release);
    animation_setter!(cancel);
    animation_setter!(cancel_reset);

    pub fn hover(
        &mut self,
        duration: f32,
        easing: impl Into<Option<Ease>>,
        delay: impl Into<Option<f32>>,
        loop_type: impl Into<Option<AnimationLoop>>,
    ) -> &mut StyleAnimation {
        let easing = easing.into();
        let delay = delay.into();
        let loop_type = loop_type.into();
        self.pointer_enter(duration, easing, delay, loop_type);
        self.pointer_leave(duration, easing, delay, loop_type);

        self
    }

    pub fn all(
        &mut self,
        duration: f32,
        easing: impl Into<Option<Ease>>,
        delay: impl Into<Option<f32>>,
        loop_type: impl Into<Option<AnimationLoop>>,
    ) -> &mut StyleAnimation {
        let config = AnimationConfig {
            duration,
            easing: easing.into(),
            delay: delay.into(),
            loop_type: loop_type.into(),
        };
        self.pointer_enter = Some(config);
        self.pointer_leave = Some(config);
        self.press = Some(config);
        self.release = Some(config);
        self.cancel = Some(config);
        self.cancel_reset = Some(config);

        self
    }

    fn to_tween(&self, flux_interaction: &FluxInteraction) -> Option<AnimationConfig> {
        match flux_interaction {
            FluxInteraction::None => None,
            FluxInteraction::PointerEnter => self.pointer_enter,
            FluxInteraction::PointerLeave => self.pointer_leave,
            FluxInteraction::Pressed => self.press,
            FluxInteraction::Released => self.release,
            FluxInteraction::PressCanceled => self.cancel,
            FluxInteraction::Disabled => None,
        }
    }

    pub fn lock_duration(&self, flux_interaction: &FluxInteraction) -> StopwatchLock {
        let Some(tween) = self.to_tween(flux_interaction) else {
            return StopwatchLock::None;
        };

        let loop_type = tween.loop_type();
        match loop_type {
            AnimationLoop::None => {
                StopwatchLock::Duration(Duration::from_secs_f32(tween.delay() + tween.duration))
            }
            AnimationLoop::Continous => StopwatchLock::Infinite,
            AnimationLoop::Times(n) => StopwatchLock::Duration(Duration::from_secs_f32(
                tween.delay() + (tween.duration * n.get() as f32),
            )),
            AnimationLoop::PingPongContinous => StopwatchLock::Infinite,
            AnimationLoop::PingPong(n) => StopwatchLock::Duration(Duration::from_secs_f32(
                tween.delay() + (tween.duration * n.get() as f32),
            )),
        }
    }

    pub fn update(
        &self,
        prev_state: &AnimationState,
        flux_interaction: &FluxInteraction,
        stopwatch: &FluxInteractionStopwatch,
    ) -> AnimationState {
        let mut target_style: InteractionStyle = flux_interaction.into();
        let mut tween = self.to_tween(flux_interaction);
        let mut elapsed = stopwatch.0.elapsed_secs();

        if target_style == InteractionStyle::Cancel {
            if let Some(cancel_tween) = tween {
                let cancel_tween_length = cancel_tween.duration + cancel_tween.delay();

                if elapsed >= cancel_tween_length {
                    target_style = InteractionStyle::Base;
                    tween = self.cancel_reset;
                    elapsed -= cancel_tween_length;
                }
            } else {
                target_style = InteractionStyle::Base;
                tween = self.cancel_reset;
            }
        }

        // No animation applied for the current interaction
        let Some(tween) = tween else {
            return AnimationState {
                result: AnimationResult::Hold(target_style),
                iteration: 0,
            };
        };

        let delay = tween.delay();
        let base_tween_length = tween.duration + delay;
        let tween_time = tween.duration.max(0.);
        let easing = tween.easing();

        // Includes elapsed == 0.
        if elapsed <= delay {
            return prev_state.clone();
        } else if let Some(_) = tween.loop_type {
            //TODO: Loops
        } else {
            // No looping
            if elapsed > base_tween_length {
                return AnimationState {
                    result: AnimationResult::Hold(target_style),
                    iteration: 0,
                };
            } else {
                let tween_ratio = ((elapsed - delay) / tween_time).clamp(0., 1.).ease(easing);
                match &prev_state.result {
                    AnimationResult::Hold(style) => {
                        if *style != target_style {
                            return AnimationState {
                                result: AnimationResult::Interpolate(
                                    *style,
                                    target_style,
                                    tween_ratio,
                                ),
                                iteration: 0,
                            };
                        } else {
                            return AnimationState {
                                result: AnimationResult::Hold(target_style),
                                iteration: 0,
                            };
                        }
                    }
                    AnimationResult::Interpolate(from, to, t) => {
                        if *from == target_style {
                            return AnimationState {
                                result: AnimationResult::Reverse(*from, *to, *t, *t),
                                iteration: 0,
                            };
                        } else if *to == target_style {
                            return AnimationState {
                                result: AnimationResult::Interpolate(*from, *to, tween_ratio),
                                iteration: 0,
                            };
                        } else {
                            return AnimationState {
                                result: AnimationResult::TransitionBetween {
                                    origin: *from,
                                    points: vec![(*to, *t), (target_style, tween_ratio)],
                                },
                                iteration: 0,
                            };
                        }
                    }
                    AnimationResult::Reverse(from, to, t, offset) => {
                        if *from == target_style {
                            let new_ratio = (offset - tween_ratio).max(0.);
                            if new_ratio == 0. {
                                return AnimationState {
                                    result: AnimationResult::Hold(target_style),
                                    iteration: 0,
                                };
                            } else {
                                return AnimationState {
                                    result: AnimationResult::Reverse(
                                        *from, *to, new_ratio, *offset,
                                    ),
                                    iteration: 0,
                                };
                            }
                        } else {
                            return AnimationState {
                                result: AnimationResult::TransitionBetween {
                                    origin: *from,
                                    points: vec![(*to, *t), (target_style, tween_ratio)],
                                },
                                iteration: 0,
                            };
                        }
                    }
                    AnimationResult::TransitionBetween { origin, points } => {
                        // TODO: this is not a frequent case, but consider finding workaround for allocation
                        let mut new_points = points.clone();

                        // Safe unwrap: We never remove points, only add, and we start with two points
                        let last_point = new_points.last_mut().unwrap();
                        if last_point.0 == target_style {
                            last_point.1 = tween_ratio;
                        } else {
                            new_points.push((target_style, tween_ratio));
                        }

                        return AnimationState {
                            result: AnimationResult::TransitionBetween {
                                origin: *origin,
                                points: new_points,
                            },
                            iteration: 0,
                        };
                    }
                };
            }
        }

        let progress = prev_state.clone();

        // TODO: handle iteration overflow
        progress
    }
}
