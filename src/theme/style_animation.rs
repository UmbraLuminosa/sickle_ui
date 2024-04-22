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
    // Interpolate(InteractionStyle, InteractionStyle, f32),
    // Reverse(InteractionStyle, InteractionStyle, f32, f32),
    Interpolate {
        from: InteractionStyle,
        to: InteractionStyle,
        t: f32,
        offset: f32,
    },
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
            AnimationResult::Interpolate { from, to, t, .. } => bundle
                .interaction_style(*from)
                .lerp(bundle.interaction_style(*to), *t),
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
        match flux_interaction {
            FluxInteraction::PressCanceled => {
                let cancel_lock = StyleAnimation::tween_lock_duration(self.cancel);
                let reset_lock = StyleAnimation::tween_lock_duration(self.cancel_reset);
                match (cancel_lock, reset_lock) {
                    (StopwatchLock::None, StopwatchLock::None) => StopwatchLock::None,
                    (StopwatchLock::None, StopwatchLock::Duration(_)) => reset_lock,
                    (StopwatchLock::Duration(_), StopwatchLock::None) => cancel_lock,
                    (StopwatchLock::Duration(l_duration), StopwatchLock::Duration(r_duration)) => {
                        StopwatchLock::Duration(l_duration + r_duration)
                    }
                    // Either side is infinite, let them cook
                    _ => StopwatchLock::Infinite,
                }
            }
            _ => StyleAnimation::tween_lock_duration(self.to_tween(flux_interaction)),
        }
    }

    fn tween_lock_duration(tween: Option<AnimationConfig>) -> StopwatchLock {
        let Some(tween) = tween else {
            return StopwatchLock::None;
        };

        match tween.loop_type() {
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
                                result: AnimationResult::Interpolate {
                                    from: *style,
                                    to: target_style,
                                    t: tween_ratio,
                                    offset: 0.,
                                },
                                iteration: 0,
                            };
                        } else {
                            return AnimationState {
                                result: AnimationResult::Hold(target_style),
                                iteration: 0,
                            };
                        }
                    }
                    AnimationResult::Interpolate {
                        from,
                        to,
                        t,
                        offset,
                    } => {
                        // TODO: Discover inner mathematical genius to implement inverse ease functions to recover x where ease(x) = offset
                        let base_ratio =
                            (((elapsed - delay) / tween_time) * (1. - offset)).clamp(0., 1.);
                        let tween_ratio = offset + base_ratio.ease(easing);

                        if *from == target_style {
                            return AnimationState {
                                result: AnimationResult::Interpolate {
                                    from: *to,
                                    to: *from,
                                    t: 1. - *t,
                                    offset: 1. - *t,
                                },
                                iteration: 0,
                            };
                        } else if *to == target_style {
                            return AnimationState {
                                result: AnimationResult::Interpolate {
                                    from: *from,
                                    to: *to,
                                    t: tween_ratio,
                                    offset: *offset,
                                },
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
                    AnimationResult::TransitionBetween { origin, points } => {
                        // TODO: this is not a frequent case, but consider finding workaround for allocation
                        let mut new_points = points.clone();
                        let point_count = new_points.len();

                        // Safe unwrap: We never remove points, only add, and we start with two points
                        let last_point = new_points.last_mut().unwrap();
                        let last_style = last_point.0;
                        if last_style == target_style {
                            last_point.1 = tween_ratio;
                        } else if point_count < 5 {
                            new_points.push((target_style, tween_ratio));
                        } else {
                            warn!(
                                "Transition animation step overflow occured: {:?}",
                                new_points.clone()
                            );

                            // Reset to the last two step's interpolation
                            return AnimationState {
                                result: AnimationResult::Interpolate {
                                    from: last_style,
                                    to: target_style,
                                    t: tween_ratio,
                                    offset: 0.,
                                },
                                iteration: 0,
                            };
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
