use crate::{
    ui_style::{AnimatedStyleAttribute, InteractiveStyleAttribute, StaticStyleAttribute},
    FluxInteraction, FluxInteractionStopwatch,
};

use super::{AnimationState, StyleAnimation};

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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DynamicStyleController {
    pub animation: StyleAnimation,
    current_state: AnimationState,
    dirty: bool,
}

impl DynamicStyleController {
    pub fn update(
        &mut self,
        flux_interaction: &FluxInteraction,
        stopwatch: &FluxInteractionStopwatch,
    ) {
        let new_state = self
            .animation
            .update(&self.current_state, flux_interaction, stopwatch);

        if new_state != self.current_state {
            // info!("{:?}", new_state);
            self.current_state = new_state;
            self.dirty = true;
        }
    }

    pub fn current_state(&self) -> &AnimationState {
        &self.current_state
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn copy_state_from(&mut self, other: &DynamicStyleController) {
        self.current_state = other.current_state().clone();
        self.dirty = other.dirty();
    }
}
