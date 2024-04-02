use std::num::NonZeroU8;

use bevy::prelude::*;
use sickle_math::ease::Ease;

use crate::{FluxInteraction, FluxInteractionStopwatch};

// TODO: Add support for continous animations, i.e. loop, ping-pong
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AnimationProgress {
    #[default]
    Start,
    Inbetween(f32),
    End,
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
pub struct StyleAnimationConfig {
    duration: f32,
    easing: Ease,
    delay: Option<f32>,
    loop_type: Option<AnimationLoop>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StyleAnimation {
    pointer_enter: Option<StyleAnimationConfig>,
    pointer_leave: Option<StyleAnimationConfig>,
    press: Option<StyleAnimationConfig>,
    release: Option<StyleAnimationConfig>,
    cancel: Option<StyleAnimationConfig>,
    focus: Option<StyleAnimationConfig>,
    focus_lost: Option<StyleAnimationConfig>,
}

macro_rules! animation_setter {
    ($setter:ident) => {
        pub fn $setter(
            &mut self,
            duration: f32,
            easing: Ease,
            delay: Option<f32>,
            loop_type: Option<AnimationLoop>,
        ) -> &mut StyleAnimation {
            let config = StyleAnimationConfig {
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
    animation_setter!(focus);
    animation_setter!(focus_lost);

    pub fn all(
        &mut self,
        duration: f32,
        easing: Ease,
        delay: Option<f32>,
        loop_type: Option<AnimationLoop>,
    ) -> &mut StyleAnimation {
        let config = StyleAnimationConfig {
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
        self.focus = Some(config);
        self.focus_lost = Some(config);

        self
    }

    pub fn update(
        &mut self,
        flux_interaction: &FluxInteraction,
        stopwatch: &FluxInteractionStopwatch,
    ) {
    }
}
