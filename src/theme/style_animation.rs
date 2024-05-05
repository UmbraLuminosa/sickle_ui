use std::{time::Duration, vec};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use sickle_math::{
    ease::{Ease, ValueEasing},
    lerp::Lerp,
};

use crate::{ui_style::AnimatedVals, FluxInteraction, StopwatchLock};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InteractionStyle {
    #[default]
    Enter,
    Idle,
    Hover,
    Press,
    Cancel,
    IdleAlt,
    HoverAlt,
    PressAlt,
}

impl From<FluxInteraction> for InteractionStyle {
    fn from(value: FluxInteraction) -> Self {
        match value {
            FluxInteraction::None => Self::Idle,
            FluxInteraction::PointerEnter => Self::Hover,
            FluxInteraction::PointerLeave => Self::Idle,
            FluxInteraction::Pressed => Self::Press,
            FluxInteraction::Released => Self::Hover,
            FluxInteraction::PressCanceled => Self::Cancel,
            FluxInteraction::Disabled => Self::Idle,
        }
    }
}

impl From<&FluxInteraction> for InteractionStyle {
    fn from(value: &FluxInteraction) -> Self {
        Self::from(*value)
    }
}

impl InteractionStyle {
    fn alt(&self) -> Option<InteractionStyle> {
        match self {
            InteractionStyle::Idle => InteractionStyle::IdleAlt.into(),
            InteractionStyle::Hover => InteractionStyle::HoverAlt.into(),
            InteractionStyle::Press => InteractionStyle::PressAlt.into(),
            InteractionStyle::Cancel => None,
            InteractionStyle::IdleAlt => InteractionStyle::Idle.into(),
            InteractionStyle::HoverAlt => InteractionStyle::Hover.into(),
            InteractionStyle::PressAlt => InteractionStyle::Press.into(),
            InteractionStyle::Enter => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationResult {
    Hold(InteractionStyle),
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
    pub fn extract<T: Lerp + Default + Clone + PartialEq>(&self, bundle: &AnimatedVals<T>) -> T {
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum AnimationLoop {
    #[default]
    None,
    Continous,
    /// Repeat animation u8 number of times. Once the animation is completed
    /// it will reset to the start value if the second argument is set to true.
    Times(u8, bool),
    PingPongContinous,
    PingPong(u8),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect, Serialize, Deserialize)]
pub struct AnimationConfig {
    pub duration: f32,
    #[reflect(default)]
    pub easing: Option<Ease>,
    #[reflect(default)]
    pub delay: Option<f32>,
}

impl AnimationConfig {
    pub fn new(
        duration: f32,
        easing: impl Into<Option<Ease>>,
        delay: impl Into<Option<f32>>,
    ) -> AnimationConfig {
        AnimationConfig {
            duration,
            easing: easing.into(),
            delay: delay.into(),
        }
    }

    pub fn delay(&self) -> f32 {
        match self.delay {
            Some(delay) => delay,
            None => 0.,
        }
    }

    pub fn easing(&self) -> Ease {
        match self.easing {
            Some(ease) => ease,
            None => Ease::Linear,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect, Serialize, Deserialize)]
pub struct LoopedAnimationConfig {
    pub duration: f32,
    #[reflect(default)]
    pub easing: Option<Ease>,
    #[reflect(default)]
    pub start_delay: Option<f32>,
    #[reflect(default)]
    pub loop_gap: Option<f32>,
    #[reflect(default)]
    pub loop_type: Option<AnimationLoop>,
}

impl LoopedAnimationConfig {
    pub fn new(
        duration: f32,
        easing: impl Into<Option<Ease>>,
        start_delay: impl Into<Option<f32>>,
        loop_gap: impl Into<Option<f32>>,
        loop_type: impl Into<Option<AnimationLoop>>,
    ) -> LoopedAnimationConfig {
        LoopedAnimationConfig {
            duration,
            easing: easing.into(),
            start_delay: start_delay.into(),
            loop_gap: loop_gap.into(),
            loop_type: loop_type.into(),
        }
    }

    fn start_delay(&self) -> f32 {
        match self.start_delay {
            Some(delay) => delay,
            None => 0.,
        }
    }

    fn loop_gap(&self) -> f32 {
        match self.loop_gap {
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

    fn is_pingpong(&self) -> bool {
        match self.loop_type() {
            AnimationLoop::PingPong(_) | AnimationLoop::PingPongContinous => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect, Serialize, Deserialize)]
pub struct AnimationSettings {
    #[reflect(default)]
    pub enter: Option<AnimationConfig>,
    #[reflect(default)]
    pub non_interacted: Option<AnimationConfig>,
    #[reflect(default)]
    pub pointer_enter: Option<AnimationConfig>,
    #[reflect(default)]
    pub pointer_leave: Option<AnimationConfig>,
    #[reflect(default)]
    pub press: Option<AnimationConfig>,
    #[reflect(default)]
    pub release: Option<AnimationConfig>,
    #[reflect(default)]
    pub cancel: Option<AnimationConfig>,
    #[reflect(default)]
    pub cancel_reset: Option<AnimationConfig>,
    #[reflect(default)]
    pub disable: Option<AnimationConfig>,
    #[reflect(default)]
    pub idle: Option<LoopedAnimationConfig>,
    #[reflect(default)]
    pub hover: Option<LoopedAnimationConfig>,
    #[reflect(default)]
    pub pressed: Option<LoopedAnimationConfig>,
}

macro_rules! transition_animation_setter {
    ($setter:ident) => {
        pub fn $setter(
            &mut self,
            duration: f32,
            easing: impl Into<Option<Ease>>,
            delay: impl Into<Option<f32>>,
        ) -> &mut Self {
            let config = AnimationConfig {
                duration,
                easing: easing.into(),
                delay: delay.into(),
            };
            self.$setter = Some(config);

            self
        }
    };
}

macro_rules! transition_from_animation_setter {
    ($setter:ident, $setter_from:ident) => {
        pub fn $setter_from(&mut self, config: impl Into<Option<AnimationConfig>>) -> &mut Self {
            self.$setter = config.into();

            self
        }
    };
}

macro_rules! state_animation_setter {
    ($setter:ident) => {
        pub fn $setter(
            &mut self,
            duration: f32,
            easing: impl Into<Option<Ease>>,
            start_delay: impl Into<Option<f32>>,
            loop_gap: impl Into<Option<f32>>,
            loop_type: impl Into<Option<AnimationLoop>>,
        ) -> &mut Self {
            if duration <= 0. {
                warn!("Invalid animation duration used: {}", duration);
            }

            let config = LoopedAnimationConfig {
                duration,
                easing: easing.into(),
                start_delay: start_delay.into(),
                loop_gap: loop_gap.into(),
                loop_type: loop_type.into(),
            };
            self.$setter = Some(config);

            self
        }
    };
}

macro_rules! state_from_animation_setter {
    ($setter:ident, $setter_from:ident) => {
        pub fn $setter_from(
            &mut self,
            config: impl Into<Option<LoopedAnimationConfig>>,
        ) -> &mut Self {
            if let Some(config) = config.into() {
                if config.duration <= 0. {
                    warn!("Invalid animation duration used: {}", config.duration);
                }
                self.$setter = Some(config);
            } else {
                self.$setter = None;
            }

            self
        }
    };
}

impl AnimationSettings {
    pub fn new() -> Self {
        Self { ..default() }
    }

    pub fn copy_from(&mut self, other: Self) -> &mut Self {
        self.enter = other.enter;
        self.non_interacted = other.non_interacted;
        self.pointer_enter = other.pointer_enter;
        self.pointer_leave = other.pointer_leave;
        self.press = other.press;
        self.release = other.release;
        self.cancel = other.cancel;
        self.cancel_reset = other.cancel_reset;
        self.disable = other.disable;
        self.idle = other.idle;
        self.hover = other.hover;
        self.pressed = other.pressed;

        self
    }

    transition_animation_setter!(enter);
    transition_animation_setter!(non_interacted);
    transition_from_animation_setter!(non_interacted, non_interacted_from);
    transition_animation_setter!(pointer_enter);
    transition_from_animation_setter!(pointer_enter, pointer_enter_from);
    transition_animation_setter!(pointer_leave);
    transition_from_animation_setter!(pointer_leave, pointer_leave_from);
    transition_animation_setter!(press);
    transition_from_animation_setter!(press, press_from);
    transition_animation_setter!(release);
    transition_from_animation_setter!(release, release_from);
    transition_animation_setter!(cancel);
    transition_from_animation_setter!(cancel, cancel_from);
    transition_animation_setter!(cancel_reset);
    transition_from_animation_setter!(cancel_reset, cancel_reset_from);
    transition_animation_setter!(disable);
    transition_from_animation_setter!(disable, disable_from);
    state_animation_setter!(idle);
    state_from_animation_setter!(idle, idle_from);
    state_animation_setter!(hover);
    state_from_animation_setter!(hover, hover_from);
    state_animation_setter!(pressed);
    state_from_animation_setter!(pressed, pressed_from);

    pub fn to_tween(&self, flux_interaction: &FluxInteraction) -> Option<AnimationConfig> {
        match flux_interaction {
            FluxInteraction::None => self.enter,
            FluxInteraction::PointerEnter => self.pointer_enter,
            FluxInteraction::PointerLeave => self.pointer_leave,
            FluxInteraction::Pressed => self.press,
            FluxInteraction::Released => self.release,
            FluxInteraction::PressCanceled => self.cancel,
            FluxInteraction::Disabled => self.disable,
        }
    }

    pub fn to_loop_tween(
        &self,
        flux_interaction: &FluxInteraction,
    ) -> Option<LoopedAnimationConfig> {
        match flux_interaction {
            FluxInteraction::None => None,
            FluxInteraction::PointerEnter => self.hover,
            FluxInteraction::PointerLeave => self.idle,
            FluxInteraction::Pressed => self.pressed,
            FluxInteraction::Released => self.idle,
            FluxInteraction::PressCanceled => self.idle,
            FluxInteraction::Disabled => None,
        }
    }

    pub fn enter_duration(&self) -> StopwatchLock {
        AnimationSettings::transition_lock_duration(self.enter)
    }

    pub fn lock_duration(&self, flux_interaction: &FluxInteraction) -> StopwatchLock {
        let transition = match flux_interaction {
            FluxInteraction::PressCanceled => {
                let cancel_lock = AnimationSettings::transition_lock_duration(self.cancel);
                let reset_lock = AnimationSettings::transition_lock_duration(self.cancel_reset);
                cancel_lock + reset_lock
            }
            _ => AnimationSettings::transition_lock_duration(self.to_tween(flux_interaction)),
        };

        let state_animation = match flux_interaction {
            FluxInteraction::None => StopwatchLock::None,
            FluxInteraction::PointerEnter => AnimationSettings::state_lock_duration(self.hover),
            FluxInteraction::PointerLeave => AnimationSettings::state_lock_duration(self.idle),
            FluxInteraction::Pressed => AnimationSettings::state_lock_duration(self.pressed),
            FluxInteraction::Released => AnimationSettings::state_lock_duration(self.idle),
            FluxInteraction::PressCanceled => AnimationSettings::state_lock_duration(self.idle),
            FluxInteraction::Disabled => StopwatchLock::None,
        };

        transition + state_animation
    }

    pub fn transition_lock_duration(tween: Option<AnimationConfig>) -> StopwatchLock {
        let Some(tween) = tween else {
            return StopwatchLock::None;
        };

        StopwatchLock::Duration(Duration::from_secs_f32(tween.delay() + tween.duration))
    }

    pub fn state_lock_duration(tween: Option<LoopedAnimationConfig>) -> StopwatchLock {
        let Some(tween) = tween else {
            return StopwatchLock::None;
        };

        match tween.loop_type() {
            AnimationLoop::None => StopwatchLock::Duration(Duration::from_secs_f32(
                tween.start_delay() + tween.duration,
            )),
            AnimationLoop::Continous => StopwatchLock::Infinite,
            AnimationLoop::Times(n, _) => StopwatchLock::Duration(Duration::from_secs_f32(
                tween.start_delay() + (tween.duration * n as f32) + (tween.loop_gap() * n as f32),
            )),
            AnimationLoop::PingPongContinous => StopwatchLock::Infinite,
            AnimationLoop::PingPong(n) => StopwatchLock::Duration(Duration::from_secs_f32(
                tween.start_delay() + (tween.duration * n as f32) + (tween.loop_gap() * n as f32),
            )),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationState {
    result: AnimationResult,
    iteration: u8,
}

impl AnimationState {
    pub fn extract<T: Lerp + Default + Clone + PartialEq>(&self, bundle: &AnimatedVals<T>) -> T {
        self.result.extract(bundle)
    }

    pub fn is_entering(&self) -> bool {
        match self.result {
            AnimationResult::Hold(style) => style == InteractionStyle::Enter,
            AnimationResult::Interpolate { from, .. } => from == InteractionStyle::Enter,
            _ => false,
        }
    }

    pub fn tick(
        &self,
        target_style: InteractionStyle,
        tween: Option<AnimationConfig>,
        loop_tween: Option<LoopedAnimationConfig>,
        elapsed: f32,
    ) -> Self {
        // No animation applied for the current interaction
        let Some(tween) = tween else {
            return AnimationState {
                result: AnimationResult::Hold(target_style),
                iteration: 0,
            };
        };

        let delay = tween.delay();
        let tween_time = tween.duration.max(0.);
        let easing = tween.easing();

        // Includes elapsed == 0.
        if elapsed <= delay {
            match &self.result {
                AnimationResult::Interpolate { from, to, t, .. } => {
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
                    }
                    return self.clone();
                }
                _ => return self.clone(),
            }
        }

        if elapsed > (tween_time + delay) {
            // Do loop or hold
            let (Some(alt_tween), Some(alt_target)) = (loop_tween, target_style.alt()) else {
                return AnimationState {
                    result: AnimationResult::Hold(target_style),
                    iteration: 0,
                };
            };

            AnimationState::process_animation_loops(
                target_style,
                alt_target,
                elapsed - (tween_time + delay),
                alt_tween,
            )
        } else {
            AnimationState::process_transition_animations(
                target_style,
                elapsed,
                delay,
                tween_time,
                easing,
                &self.result,
            )
        }
    }

    fn process_animation_loops(
        target_style: InteractionStyle,
        alt_target: InteractionStyle,
        mut elapsed: f32,
        tween: LoopedAnimationConfig,
    ) -> AnimationState {
        let start_delay = tween.start_delay();
        if tween.loop_type() == AnimationLoop::None || elapsed < start_delay || tween.duration <= 0.
        {
            return AnimationState {
                result: AnimationResult::Hold(target_style),
                iteration: 0,
            };
        }
        elapsed -= start_delay;

        let loop_gap = tween.loop_gap();
        let iteration = (elapsed / (tween.duration + loop_gap)).floor() as usize;
        let even = iteration % 2 == 0;

        match tween.loop_type() {
            AnimationLoop::Times(times, reset) => {
                if iteration >= times as usize {
                    return AnimationState {
                        result: AnimationResult::Hold(match reset {
                            true => target_style,
                            false => alt_target,
                        }),
                        iteration: (iteration % 255) as u8,
                    };
                }
            }
            AnimationLoop::PingPong(times) => {
                if iteration >= times as usize {
                    return AnimationState {
                        result: AnimationResult::Hold(match even {
                            true => target_style,
                            false => alt_target,
                        }),
                        iteration: (iteration % 255) as u8,
                    };
                }
            }
            _ => (),
        }

        let offset = elapsed % (tween.duration + loop_gap);
        if loop_gap > 0. && offset > tween.duration {
            // We are in the pause-gap
            let hold_style = match tween.is_pingpong() {
                true => match even {
                    true => alt_target,
                    false => target_style,
                },
                false => alt_target,
            };

            AnimationState {
                result: AnimationResult::Hold(hold_style),
                iteration: (iteration % 255) as u8,
            }
        } else {
            let tween_ratio = (offset / tween.duration).clamp(0., 1.).ease(tween.easing());
            let from = match tween.is_pingpong() {
                true => match even {
                    true => target_style,
                    false => alt_target,
                },
                false => target_style,
            };
            let to = match tween.is_pingpong() {
                true => match even {
                    true => alt_target,
                    false => target_style,
                },
                false => alt_target,
            };

            AnimationState {
                result: AnimationResult::Interpolate {
                    from,
                    to,
                    t: tween_ratio,
                    offset: 0.,
                },
                iteration: (iteration % 255) as u8,
            }
        }
    }

    fn process_transition_animations(
        target_style: InteractionStyle,
        elapsed: f32,
        delay: f32,
        tween_time: f32,
        easing: Ease,
        previous_result: &AnimationResult,
    ) -> AnimationState {
        let tween_ratio = ((elapsed - delay) / tween_time).clamp(0., 1.).ease(easing);
        match previous_result {
            AnimationResult::Hold(prev_style) => {
                AnimationState::process_hold(target_style, prev_style, tween_ratio)
            }
            AnimationResult::Interpolate {
                from,
                to,
                t,
                offset,
            } => AnimationState::process_interpolate(
                target_style,
                elapsed,
                delay,
                tween_time,
                easing,
                from,
                to,
                t,
                offset,
            ),
            AnimationResult::TransitionBetween { origin, points } => {
                AnimationState::process_transition_between(
                    target_style,
                    tween_ratio,
                    origin,
                    points,
                )
            }
        }
    }

    fn process_hold(
        target_style: InteractionStyle,
        prev_style: &InteractionStyle,
        tween_ratio: f32,
    ) -> AnimationState {
        if *prev_style != target_style {
            AnimationState {
                result: AnimationResult::Interpolate {
                    from: *prev_style,
                    to: target_style,
                    t: tween_ratio,
                    offset: 0.,
                },
                iteration: 0,
            }
        } else {
            AnimationState {
                result: AnimationResult::Hold(target_style),
                iteration: 0,
            }
        }
    }

    fn process_interpolate(
        target_style: InteractionStyle,
        elapsed: f32,
        delay: f32,
        tween_time: f32,
        easing: Ease,
        from: &InteractionStyle,
        to: &InteractionStyle,
        t: &f32,
        offset: &f32,
    ) -> AnimationState {
        // Best effort complete the animation by tweening for only the remaining distance.
        // We could store `elapsed` and the `easing` type and try to continue animations,
        // but there is no guarantee we continue the interrupted one.
        let base_ratio = (((elapsed - delay) / tween_time) * (1. - offset)).clamp(0., 1.);
        let tween_ratio = offset + base_ratio.ease(easing);

        if *from == target_style {
            AnimationState {
                result: AnimationResult::Interpolate {
                    from: *to,
                    to: *from,
                    t: 1. - *t,
                    offset: 1. - *t,
                },
                iteration: 0,
            }
        } else if *to == target_style {
            AnimationState {
                result: AnimationResult::Interpolate {
                    from: *from,
                    to: *to,
                    t: tween_ratio,
                    offset: *offset,
                },
                iteration: 0,
            }
        } else {
            AnimationState {
                result: AnimationResult::TransitionBetween {
                    origin: *from,
                    points: vec![(*to, *t), (target_style, tween_ratio)],
                },
                iteration: 0,
            }
        }
    }

    fn process_transition_between(
        target_style: InteractionStyle,
        tween_ratio: f32,
        origin: &InteractionStyle,
        points: &Vec<(InteractionStyle, f32)>,
    ) -> AnimationState {
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

            // At this point, this is from a weird jiggle, escape leak!
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

        AnimationState {
            result: AnimationResult::TransitionBetween {
                origin: *origin,
                points: new_points,
            },
            iteration: 0,
        }
    }
}
