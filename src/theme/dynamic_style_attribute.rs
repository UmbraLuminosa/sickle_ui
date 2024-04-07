use crate::{
    ui_style::{
        AnimatedStyleAttribute, InteractionAnimationState, InteractiveStyleAttribute,
        StaticStyleAttribute,
    },
    FluxInteraction, FluxInteractionStopwatch,
};

use super::{AnimationProgress, StyleAnimation};

/*
DynamicStyle::build(|builder| {
    builder
        .inert() // InertDynamicStyleBuilder
        .background_color(Color::BLUE)
        .border_color(Color::BLUE);
    builder
        .interactive() // InteractiveDynamicStyleBuilder
        // ideally cached as part of theming commons
        .background_color(AttributeBundle<Color>{ Color::BLUE, Color::RED.into(), None, None, None });
    builder
        .animated() // AnimatedDynamicStyleBuilder
        .background_color(LerpAttributeBundle<Color>{ Color::BLUE, Color::RED.into(), None, None, None })
        .pointer_enter(0.2, Ease::InOutExpo, None, LoopType::Continous)
        .pointer_leave(0.2, Ease::InOutExpo, Some(0.2), None);
});

DynamicAttribute:
- Inert values:
  - No update after initial setting
  - Any stylable attribute
  - Custom inert value:
    - Single callback
- Flux-static values
  - Update per flux status change
  - Any stylable attribute
  - Custom flux-static values:
    - Callback with flux state
- Animated interactive values
  - Update per frame
  - Potentially locking stopwatch
  - Any stylable Lerp attribute
  - Custom animated values:
    - Callback with flux state and animation progress / loop

// apply() returns lock length (None, f32, Indefinite)
// -> eval flux lock length on flux interaction change (at DynamicStyle level, iterate over all Animated)
// TODO: lock cleanup automatically on flux interaction change -> part of flux interaction system stack
// FluxStopwatchLock? Merge lock lengths?
*/

#[derive(Clone, Debug, PartialEq)]
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
        let (progress, _) = self.animation.to_progress(flux_interaction, stopwatch);
        if self.transition_base.progress != progress {
            match *flux_interaction {
                FluxInteraction::PointerEnter => {
                    self.transition_base = InteractionAnimationState {
                        progress,
                        iteration: 0,
                        phase: FluxInteraction::PointerEnter,
                    }
                }
                FluxInteraction::Released => {
                    self.transition_base = InteractionAnimationState {
                        progress: AnimationProgress::End,
                        iteration: 0,
                        phase: FluxInteraction::PointerEnter,
                    }
                }
                _ => (),
            }
        }

        if self.current_state.phase != *flux_interaction || self.current_state.progress != progress
        {
            self.current_state = InteractionAnimationState {
                progress,
                iteration: 0,
                phase: *flux_interaction,
            };
            self.dirty = true;
        } else {
            self.dirty = false;
        }
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
