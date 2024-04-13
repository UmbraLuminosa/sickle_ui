use std::num::NonZeroU8;

use bevy::prelude::*;
use sickle_math::ease::{Ease, ValueEasing};

use crate::{FluxInteraction, FluxInteractionStopwatch};

// TODO: Add support for continous animations, i.e. loop, ping-pong
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationProgress {
    Start(FluxInteraction),
    Inbetween(FluxInteraction, FluxInteraction, f32),
    End(FluxInteraction),
}

impl Default for AnimationProgress {
    fn default() -> Self {
        Self::Start(FluxInteraction::None)
    }
}

impl AnimationProgress {
    pub fn is_start(&self) -> bool {
        matches!(self, AnimationProgress::Start(_))
    }

    pub fn is_inbetween(&self) -> bool {
        matches!(self, AnimationProgress::Inbetween(_, _, _))
    }

    pub fn is_end(&self) -> bool {
        matches!(self, AnimationProgress::End(_))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AnimationLoop {
    #[default]
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StyleAnimation {
    pointer_enter: Option<AnimationConfig>,
    pointer_leave: Option<AnimationConfig>,
    press: Option<AnimationConfig>,
    release: Option<AnimationConfig>,
    cancel: Option<AnimationConfig>,
    cancel_reset: Option<AnimationConfig>,
    focus: Option<AnimationConfig>,
    focus_lost: Option<AnimationConfig>,
}

macro_rules! animation_setter {
    ($setter:ident) => {
        pub fn $setter(
            &mut self,
            duration: f32,
            easing: Option<Ease>,
            delay: Option<f32>,
            loop_type: Option<AnimationLoop>,
        ) -> &mut StyleAnimation {
            let config = AnimationConfig {
                duration,
                easing,
                delay,
                loop_type,
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
    animation_setter!(focus);
    animation_setter!(focus_lost);

    pub fn hover(
        &mut self,
        duration: f32,
        easing: Option<Ease>,
        delay: Option<f32>,
        loop_type: Option<AnimationLoop>,
    ) -> &mut StyleAnimation {
        self.pointer_enter(duration, easing, delay, loop_type);
        self.pointer_leave(duration, easing, delay, loop_type);

        self
    }

    pub fn all(
        &mut self,
        duration: f32,
        easing: Option<Ease>,
        delay: Option<f32>,
        loop_type: Option<AnimationLoop>,
    ) -> &mut StyleAnimation {
        let config = AnimationConfig {
            duration,
            easing,
            delay,
            loop_type,
        };
        self.pointer_enter = Some(config);
        self.pointer_leave = Some(config);
        self.press = Some(config);
        self.release = Some(config);
        self.cancel = Some(config);
        self.cancel_reset = Some(config);
        self.focus = Some(config);
        self.focus_lost = Some(config);

        self
    }

    pub fn to_progress(
        &self,
        flux_interaction: &FluxInteraction,
        stopwatch: &FluxInteractionStopwatch,
    ) -> (AnimationProgress, bool) {
        let elapsed = stopwatch.0.elapsed_secs();

        let progress = match *flux_interaction {
            FluxInteraction::Pressed => self.calculate_progress(
                self.press,
                FluxInteraction::PointerEnter,
                FluxInteraction::Pressed,
                elapsed,
            ),
            FluxInteraction::Released => self.calculate_progress(
                self.release,
                FluxInteraction::Pressed,
                FluxInteraction::PointerEnter,
                elapsed,
            ),
            FluxInteraction::PressCanceled => self.calculate_cancel_progress(elapsed),
            FluxInteraction::PointerEnter => self.calculate_progress(
                self.pointer_enter,
                FluxInteraction::None,
                FluxInteraction::PointerEnter,
                elapsed,
            ),
            FluxInteraction::PointerLeave => self.calculate_progress(
                self.pointer_leave,
                FluxInteraction::PointerEnter,
                FluxInteraction::PointerLeave,
                elapsed,
            ),
            _ => AnimationProgress::End(*flux_interaction),
        };

        // TODO: New state and overflow flag
        (progress, false)
    }

    fn calculate_cancel_progress(&self, elapsed: f32) -> AnimationProgress {
        let cancel_progress = self.calculate_progress(
            self.cancel,
            FluxInteraction::Pressed,
            FluxInteraction::PressCanceled,
            elapsed,
        );
        if cancel_progress.is_end() {
            let cancel_duration = match self.cancel {
                Some(tween) => {
                    tween.duration
                        + match tween.delay {
                            Some(delay) => delay,
                            None => 0.,
                        }
                }
                None => 0.,
            };

            return self.calculate_progress(
                self.cancel_reset,
                FluxInteraction::PressCanceled,
                FluxInteraction::None,
                elapsed - cancel_duration,
            );
        }

        cancel_progress
    }

    fn calculate_progress(
        &self,
        tween: Option<AnimationConfig>,
        start_phase: FluxInteraction,
        end_phase: FluxInteraction,
        elapsed: f32,
    ) -> AnimationProgress {
        if let Some(tween) = tween {
            let tween_time = tween.duration.max(0.);
            let easing = match tween.easing {
                Some(ease) => ease,
                None => Ease::Linear,
            };

            // TODO: deal with loops

            if let Some(delay) = tween.delay {
                if elapsed < delay {
                    AnimationProgress::Start(start_phase)
                } else {
                    // Jump to end if duration is zero
                    if tween_time == 0. {
                        AnimationProgress::End(end_phase)
                    } else {
                        let tween_ratio =
                            ((elapsed - delay) / tween_time).clamp(0., 1.).ease(easing);
                        if tween_ratio == 1. {
                            AnimationProgress::End(end_phase)
                        } else {
                            AnimationProgress::Inbetween(start_phase, end_phase, tween_ratio)
                        }
                    }
                }
            } else {
                // Jump to end if duration is zero
                if tween_time == 0. {
                    AnimationProgress::End(end_phase)
                } else {
                    let tween_ratio = (elapsed / tween_time).clamp(0., 1.).ease(easing);
                    if tween_ratio == 1. {
                        AnimationProgress::End(end_phase)
                    } else {
                        AnimationProgress::Inbetween(start_phase, end_phase, tween_ratio)
                    }
                }
            }
        } else {
            AnimationProgress::End(end_phase)
        }
    }
}
