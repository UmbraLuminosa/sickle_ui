mod attribute;
mod builder;
mod generated;
mod manual;

use bevy::{
    ecs::system::{EntityCommand, EntityCommands},
    prelude::*,
    text::TextLayoutInfo,
    ui::{widget::TextFlags, FocusPolicy},
    utils::HashSet,
};
use serde::{Deserialize, Serialize};

use sickle_math::lerp::Lerp;

use crate::theme::{
    dynamic_style::{ContextStyleAttribute, DynamicStyle},
    dynamic_style_attribute::{DynamicStyleAttribute, DynamicStyleController},
    icons::IconData,
    style_animation::{AnimationSettings, AnimationState, InteractionStyle},
    typography::SizedFont,
    UiContext,
};

pub use crate::FluxInteraction;

use std::{
    fmt::{Debug, Formatter, Result},
    sync::Arc,
};

// TODO: Reorganize imports / expprts
pub use attribute::*;
pub use builder::*;
pub use generated::*;
pub use manual::*;

pub struct UiStyle<'a> {
    commands: EntityCommands<'a>,
}

impl<'a> UiStyle<'a> {
    pub fn id(&self) -> Entity {
        self.commands.id()
    }

    pub fn entity_commands(&mut self) -> EntityCommands {
        self.commands.reborrow()
    }
}

pub trait UiStyleExt<'a> {
    fn style(&'a mut self, entity: Entity) -> UiStyle<'a>;
}

impl<'a> UiStyleExt<'a> for Commands<'_, '_> {
    fn style(&'a mut self, entity: Entity) -> UiStyle<'a> {
        UiStyle {
            commands: self.entity(entity),
        }
    }
}

pub struct UiStyleUnchecked<'a> {
    commands: EntityCommands<'a>,
}

impl<'a> UiStyleUnchecked<'a> {
    pub fn id(&self) -> Entity {
        self.commands.id()
    }

    pub fn entity_commands(&mut self) -> EntityCommands {
        self.commands.reborrow()
    }
}

pub trait UiStyleUncheckedExt<'a> {
    fn style_unchecked(&'a mut self, entity: Entity) -> UiStyleUnchecked<'a>;
}

impl<'a> UiStyleUncheckedExt<'a> for Commands<'_, '_> {
    fn style_unchecked(&'a mut self, entity: Entity) -> UiStyleUnchecked<'a> {
        UiStyleUnchecked {
            commands: self.entity(entity),
        }
    }
}

pub trait LogicalEq<Rhs: ?Sized = Self> {
    fn logical_eq(&self, other: &Rhs) -> bool;

    fn logical_ne(&self, other: &Rhs) -> bool {
        !self.logical_eq(other)
    }
}

#[derive(Component, Debug, Default, Reflect)]
pub struct LockedStyleAttributes(HashSet<LockableStyleAttribute>);

impl LockedStyleAttributes {
    pub fn new() -> Self {
        Self(HashSet::<LockableStyleAttribute>::new())
    }
    pub fn lock(attributes: impl Into<HashSet<LockableStyleAttribute>>) -> Self {
        Self(attributes.into())
    }

    pub fn from_vec(attributes: Vec<LockableStyleAttribute>) -> Self {
        let mut set = HashSet::<LockableStyleAttribute>::with_capacity(attributes.len());
        for attribute in attributes.iter() {
            if !set.contains(attribute) {
                set.insert(*attribute);
            }
        }

        Self(set)
    }

    pub fn contains(&self, attr: LockableStyleAttribute) -> bool {
        self.0.contains(&attr)
    }
}

impl From<LockableStyleAttribute> for HashSet<LockableStyleAttribute> {
    fn from(value: LockableStyleAttribute) -> Self {
        let mut set = HashSet::<LockableStyleAttribute>::new();
        set.insert(value);
        set
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum TrackedStyleState {
    #[default]
    None,
    Transitioning,
    Enter,
    Idle,
    Hover,
    Pressed,
    Released,
    Canceled,
}

impl Lerp for TrackedStyleState {
    fn lerp(&self, to: Self, t: f32) -> Self {
        if t == 0. {
            *self
        } else if t == 1. {
            to
        } else {
            Self::Transitioning
        }
    }
}

impl TrackedStyleState {
    pub fn default_vals() -> AnimatedVals<Self> {
        AnimatedVals {
            idle: Self::Idle,
            hover: Self::Hover.into(),
            press: Self::Pressed.into(),
            cancel: Self::Canceled.into(),
            enter_from: Self::Enter.into(),
            ..default()
        }
    }
}
