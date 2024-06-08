use std::{
    fmt::{Debug, Formatter, Result},
    sync::Arc,
};

use bevy::{ecs::system::EntityCommand, prelude::*};
use serde::{Deserialize, Serialize};

use sickle_math::lerp::Lerp;

use crate::{flux_interaction::FluxInteraction, theme::prelude::*};

#[derive(Clone, Copy, Debug, Default, Reflect, Serialize, Deserialize)]
pub struct InteractiveVals<T: Clone + Default> {
    pub idle: T,
    #[reflect(default)]
    pub hover: Option<T>,
    #[reflect(default)]
    pub press: Option<T>,
    #[reflect(default)]
    pub cancel: Option<T>,
}

impl<T: Default + Clone> From<T> for InteractiveVals<T> {
    fn from(value: T) -> Self {
        InteractiveVals::new(value)
    }
}

impl<T: Default + Clone + PartialEq> PartialEq for InteractiveVals<T> {
    fn eq(&self, other: &Self) -> bool {
        self.idle == other.idle
            && self.hover == other.hover
            && self.press == other.press
            && self.cancel == other.cancel
    }
}

impl<T: Clone + Default> InteractiveVals<T> {
    pub fn new(value: T) -> Self {
        InteractiveVals {
            idle: value,
            ..default()
        }
    }

    pub fn hover(self, value: T) -> Self {
        Self {
            hover: value.into(),
            ..self
        }
    }

    pub fn press(self, value: T) -> Self {
        Self {
            press: value.into(),
            ..self
        }
    }

    pub fn cancel(self, value: T) -> Self {
        Self {
            cancel: value.into(),
            ..self
        }
    }

    pub fn to_value(&self, flux_interaction: FluxInteraction) -> T {
        match flux_interaction {
            FluxInteraction::None => self.idle.clone(),
            FluxInteraction::PointerEnter => self.hover.clone().unwrap_or(self.idle.clone()),
            FluxInteraction::PointerLeave => self.idle.clone(),
            FluxInteraction::Pressed => self
                .press
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.idle.clone())),
            FluxInteraction::Released => self.hover.clone().unwrap_or(self.idle.clone()),
            FluxInteraction::PressCanceled => self.cancel.clone().unwrap_or(self.idle.clone()),
            FluxInteraction::Disabled => self.idle.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Reflect, Serialize, Deserialize)]
pub struct AnimatedVals<T: Lerp + Default + Clone + PartialEq> {
    pub idle: T,
    #[reflect(default)]
    pub hover: Option<T>,
    #[reflect(default)]
    pub press: Option<T>,
    #[reflect(default)]
    pub cancel: Option<T>,
    #[reflect(default)]
    pub idle_alt: Option<T>,
    #[reflect(default)]
    pub hover_alt: Option<T>,
    #[reflect(default)]
    pub press_alt: Option<T>,
    #[reflect(default)]
    pub enter_from: Option<T>,
}

impl<T: Lerp + Default + Clone + PartialEq> From<T> for AnimatedVals<T> {
    fn from(value: T) -> Self {
        AnimatedVals {
            idle: value,
            ..default()
        }
    }
}

impl<T: Lerp + Default + Clone + PartialEq> From<InteractiveVals<T>> for AnimatedVals<T> {
    fn from(value: InteractiveVals<T>) -> Self {
        Self {
            idle: value.idle,
            hover: value.hover,
            press: value.press,
            cancel: value.cancel,
            ..default()
        }
    }
}

impl<T: Lerp + Default + Clone + PartialEq> AnimatedVals<T> {
    pub fn interaction_style(&self, interaction: InteractionStyle) -> T {
        match interaction {
            InteractionStyle::Idle => self.idle.clone(),
            InteractionStyle::Hover => self.hover.clone().unwrap_or(self.idle.clone()),
            InteractionStyle::Press => self
                .press
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.idle.clone())),
            InteractionStyle::Cancel => self.cancel.clone().unwrap_or(self.idle.clone()),
            InteractionStyle::IdleAlt => self
                .idle_alt
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.idle.clone())),
            InteractionStyle::HoverAlt => self.hover_alt.clone().unwrap_or(self.idle.clone()),
            InteractionStyle::PressAlt => self
                .press_alt
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.idle.clone())),
            InteractionStyle::Enter => self.enter_from.clone().unwrap_or(self.idle.clone()),
        }
    }

    pub fn to_value(&self, current_state: &AnimationState) -> T {
        current_state.extract(&self)
    }
}

#[derive(Clone)]
pub struct CustomStaticStyleAttribute {
    pub callback: Arc<dyn Fn(Entity, &mut World) + Send + Sync + 'static>,
}

impl CustomStaticStyleAttribute {
    pub fn new(callback: impl Fn(Entity, &mut World) + Send + Sync + 'static) -> Self {
        Self {
            callback: Arc::new(callback),
        }
    }
}

impl Debug for CustomStaticStyleAttribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("CustomStaticStyleAttribute").finish()
    }
}

impl PartialEq for CustomStaticStyleAttribute {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.callback, &other.callback)
    }
}

#[derive(Clone)]
pub struct CustomInteractiveStyleAttribute {
    pub callback: Arc<dyn Fn(Entity, FluxInteraction, &mut World) + Send + Sync + 'static>,
}

impl CustomInteractiveStyleAttribute {
    pub fn new(
        callback: impl Fn(Entity, FluxInteraction, &mut World) + Send + Sync + 'static,
    ) -> Self {
        Self {
            callback: Arc::new(callback),
        }
    }
}

impl Debug for CustomInteractiveStyleAttribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("CustomInteractiveStyleAttribute").finish()
    }
}

impl PartialEq for CustomInteractiveStyleAttribute {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.callback, &other.callback)
    }
}

#[derive(Clone)]
pub struct CustomAnimatedStyleAttribute {
    pub callback: Arc<dyn Fn(Entity, AnimationState, &mut World) + Send + Sync + 'static>,
}

impl CustomAnimatedStyleAttribute {
    pub fn new(
        callback: impl Fn(Entity, AnimationState, &mut World) + Send + Sync + 'static,
    ) -> Self {
        Self {
            callback: Arc::new(callback),
        }
    }
}

impl Debug for CustomAnimatedStyleAttribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("CustomAnimatedStyleAttribute").finish()
    }
}

impl PartialEq for CustomAnimatedStyleAttribute {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.callback, &other.callback)
    }
}

pub struct ApplyCustomStaticStyleAttribute {
    pub callback: CustomStaticStyleAttribute,
}

impl EntityCommand for ApplyCustomStaticStyleAttribute {
    fn apply(self, id: Entity, world: &mut World) {
        (self.callback.callback)(id, world);
    }
}

pub struct ApplyCustomInteractiveStyleAttribute {
    pub callback: CustomInteractiveStyleAttribute,
    pub flux_interaction: FluxInteraction,
}

impl EntityCommand for ApplyCustomInteractiveStyleAttribute {
    fn apply(self, id: Entity, world: &mut World) {
        (self.callback.callback)(id, self.flux_interaction, world);
    }
}

pub struct ApplyCustomAnimatadStyleAttribute {
    pub callback: CustomAnimatedStyleAttribute,
    pub current_state: AnimationState,
}

impl EntityCommand for ApplyCustomAnimatadStyleAttribute {
    fn apply(self, id: Entity, world: &mut World) {
        (self.callback.callback)(id, self.current_state, world);
    }
}
