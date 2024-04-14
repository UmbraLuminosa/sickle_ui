use crate::{
    ui_style::{
        AnimatedStyleAttribute, InteractionAnimationState, InteractiveStyleAttribute,
        StaticStyleAttribute,
    },
    FluxInteraction, FluxInteractionStopwatch,
};

use super::{AnimationProgress, StyleAnimation};

/*
// TODO: lock cleanup automatically on flux interaction change -> part of flux interaction system stack
// FluxStopwatchLock? Merge lock lengths?
*/

#[derive(Clone, Debug)]
pub enum DynamicStyleAttribute {
    // Remove on apply
    Static(StaticStyleAttribute),

    // Needs flux
    Interactive(InteractiveStyleAttribute),

    // Needs stopwatch
    // None animations are effectively Pop
    // Only Lerp properties
    Animated {
        attribute: AnimatedStyleAttribute,
        controller: DynamicStyleController,
    },
}

impl PartialEq for DynamicStyleAttribute {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Static(l0), Self::Static(r0)) => l0 == r0,
            (Self::Static(l0), Self::Interactive(r0)) => l0 == r0,
            (
                Self::Static(l0),
                Self::Animated {
                    attribute: r_attribute,
                    ..
                },
            ) => l0 == r_attribute,
            (Self::Interactive(l0), Self::Interactive(r0)) => l0 == r0,
            (Self::Interactive(l0), Self::Static(r0)) => l0 == r0,
            (
                Self::Interactive(l0),
                Self::Animated {
                    attribute: r_attribute,
                    ..
                },
            ) => l0 == r_attribute,
            (
                Self::Animated {
                    attribute: l_attribute,
                    ..
                },
                Self::Animated {
                    attribute: r_attribute,
                    ..
                },
            ) => l_attribute == r_attribute,
            (
                Self::Animated {
                    attribute: l_attribute,
                    ..
                },
                Self::Static(r0),
            ) => l_attribute == r0,
            (
                Self::Animated {
                    attribute: l_attribute,
                    ..
                },
                Self::Interactive(r0),
            ) => l_attribute == r0,
        }
    }
}

impl DynamicStyleAttribute {
    pub fn is_static(&self) -> bool {
        match self {
            DynamicStyleAttribute::Static(_) => true,
            _ => false,
        }
    }

    pub fn is_interactive(&self) -> bool {
        match self {
            DynamicStyleAttribute::Interactive(_) => true,
            _ => false,
        }
    }

    pub fn is_animated(&self) -> bool {
        match self {
            DynamicStyleAttribute::Animated { .. } => true,
            _ => false,
        }
    }

    pub fn controller(&self) -> Result<&DynamicStyleController, &'static str> {
        let DynamicStyleAttribute::Animated { ref controller, .. } = self else {
            return Err("DynamicStyleAttribute isn't animated!");
        };

        Ok(controller)
    }

    pub fn controller_mut(&mut self) -> Result<&mut DynamicStyleController, &'static str> {
        let DynamicStyleAttribute::Animated {
            ref mut controller, ..
        } = self
        else {
            return Err("DynamicStyleAttribute isn't animated!");
        };

        Ok(controller)
    }

    pub fn update(
        &mut self,
        flux_interaction: &FluxInteraction,
        stopwatch: &FluxInteractionStopwatch,
    ) {
        if let DynamicStyleAttribute::Animated { controller, .. } = self {
            controller.update(flux_interaction, stopwatch);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DynamicStyleController {
    pub animation: StyleAnimation,
    transition_base: InteractionAnimationState,
    current_state: InteractionAnimationState,
    dirty: bool,
}

impl DynamicStyleController {
    pub fn update(
        &mut self,
        flux_interaction: &FluxInteraction,
        stopwatch: &FluxInteractionStopwatch,
    ) {
        // TODO: track overflow
        // TODO: fix issue with animation when the cursor briefly passes over an element that has a delay on pointer_leave
        let (progress, _) = self.animation.to_progress(flux_interaction, stopwatch);
        let update_transition_base = match (self.current_state.progress, progress) {
            (AnimationProgress::Start(l_phase), AnimationProgress::Start(r_phase)) => {
                l_phase != r_phase
            }
            (AnimationProgress::End(l_phase), AnimationProgress::End(r_phase)) => {
                l_phase != r_phase
            }
            (
                AnimationProgress::Inbetween(l_start_phase, l_end_phase, _),
                AnimationProgress::Inbetween(r_start_phase, r_end_phase, _),
            ) => l_start_phase != r_start_phase || l_end_phase != r_end_phase,
            _ => true,
        };

        let update_current = match (self.current_state.progress, progress) {
            (AnimationProgress::Start(l_phase), AnimationProgress::Start(r_phase)) => {
                l_phase != r_phase
            }
            (AnimationProgress::End(l_phase), AnimationProgress::End(r_phase)) => {
                l_phase != r_phase
            }
            (
                AnimationProgress::Inbetween(l_start_phase, l_end_phase, l_t),
                AnimationProgress::Inbetween(r_start_phase, r_end_phase, r_t),
            ) => l_start_phase != r_start_phase || l_end_phase != r_end_phase || l_t != r_t,
            _ => true,
        };

        if update_transition_base {
            self.transition_base = self.current_state;
        }

        if update_current {
            self.current_state = InteractionAnimationState {
                progress,
                iteration: 0,
            };
        }

        self.dirty = update_transition_base || update_current;
    }

    pub fn transition_base(&self) -> InteractionAnimationState {
        self.transition_base
    }

    pub fn current_state(&self) -> InteractionAnimationState {
        self.current_state
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn copy_state_from(&mut self, other: &DynamicStyleController) {
        self.transition_base = other.transition_base();
        self.current_state = other.current_state();
        self.dirty = other.dirty();
    }
}
