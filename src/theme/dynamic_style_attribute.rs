use bevy::utils::default;

use crate::{
    ui_style::{
        AnimatedStyleAttribute, InteractiveStyleAttribute, LogicalEq, StaticStyleAttribute,
    },
    FluxInteraction,
};

use super::{AnimationState, InteractionStyle, AnimationSettings};

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

impl LogicalEq for DynamicStyleAttribute {
    fn logical_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Static(l0), Self::Static(r0)) => l0.logical_eq(r0),
            (Self::Static(l0), Self::Interactive(r0)) => l0.logical_eq(r0),
            (
                Self::Static(l0),
                Self::Animated {
                    attribute: r_attribute,
                    ..
                },
            ) => l0.logical_eq(r_attribute),
            (Self::Interactive(l0), Self::Interactive(r0)) => l0.logical_eq(r0),
            (Self::Interactive(l0), Self::Static(r0)) => l0.logical_eq(r0),
            (
                Self::Interactive(l0),
                Self::Animated {
                    attribute: r_attribute,
                    ..
                },
            ) => l0.logical_eq(r_attribute),
            (
                Self::Animated {
                    attribute: l_attribute,
                    ..
                },
                Self::Animated {
                    attribute: r_attribute,
                    ..
                },
            ) => l_attribute.logical_eq(r_attribute),
            (
                Self::Animated {
                    attribute: l_attribute,
                    ..
                },
                Self::Static(r0),
            ) => l_attribute.logical_eq(r0),
            (
                Self::Animated {
                    attribute: l_attribute,
                    ..
                },
                Self::Interactive(r0),
            ) => l_attribute.logical_eq(r0),
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
        matches!(self, DynamicStyleAttribute::Animated { .. })
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicStyleController {
    pub animation: AnimationSettings,
    current_state: AnimationState,
    dirty: bool,
    entering: bool,
}

impl Default for DynamicStyleController {
    fn default() -> Self {
        Self {
            animation: Default::default(),
            current_state: Default::default(),
            dirty: Default::default(),
            entering: true,
        }
    }
}

impl DynamicStyleController {
    pub fn new(animation: AnimationSettings, starting_state: AnimationState) -> Self {
        Self {
            animation,
            current_state: starting_state,
            ..default()
        }
    }

    pub fn update(&mut self, flux_interaction: &FluxInteraction, mut elapsed: f32) {
        // Pre-check if we should tick an enter frame
        let entering = match self.animation.enter.is_some() {
            true => self.entering,
            false => false,
        };

        let mut target_style: InteractionStyle = match entering {
            true => InteractionStyle::Idle,
            false => flux_interaction.into(),
        };
        let mut tween = match entering {
            true => self.animation.enter,
            false => self.animation.to_tween(&flux_interaction),
        };
        let loop_tween = match entering {
            true => None,
            false => self.animation.to_loop_tween(&flux_interaction),
        };

        if target_style == InteractionStyle::Cancel {
            if let Some(cancel_tween) = tween {
                let cancel_tween_length = cancel_tween.duration + cancel_tween.delay();

                if elapsed >= cancel_tween_length {
                    target_style = InteractionStyle::Idle;
                    tween = self.animation.cancel_reset;
                    elapsed -= cancel_tween_length;
                }
            } else {
                target_style = InteractionStyle::Idle;
                tween = self.animation.cancel_reset;
            }
        }

        let new_state = self
            .current_state
            .tick(target_style, tween, loop_tween, elapsed);

        // Remove entering flag post tick, to allow Hold to occur
        self.entering = match self.animation.enter {
            Some(tween) => self.entering && elapsed < (tween.duration + tween.delay()),
            None => false,
        };

        if new_state != self.current_state {
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

    pub fn entering(&self) -> bool {
        self.entering
    }

    pub fn copy_state_from(&mut self, other: &DynamicStyleController) {
        self.current_state = other.current_state().clone();
        self.entering = other.entering;
        self.dirty = other.dirty;
    }
}
