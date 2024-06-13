pub mod attribute;
pub mod builder;
pub mod generated;
pub mod manual;

use bevy::{ecs::system::EntityCommands, prelude::*, utils::HashSet};

use sickle_math::lerp::Lerp;

use attribute::AnimatedVals;
use generated::LockableStyleAttribute;

pub mod prelude {
    pub use super::{
        attribute::{AnimatedVals, InteractiveVals},
        builder::StyleBuilder,
        generated::*,
        manual::*,
        *,
    };
}

pub struct UiStyle<'a> {
    commands: EntityCommands<'a>,
}

impl UiStyle<'_> {
    pub fn id(&self) -> Entity {
        self.commands.id()
    }

    pub fn entity_commands(&mut self) -> EntityCommands {
        self.commands.reborrow()
    }
}

pub trait UiStyleExt {
    fn style(&mut self, entity: Entity) -> UiStyle;
}

impl UiStyleExt for Commands<'_, '_> {
    fn style(&mut self, entity: Entity) -> UiStyle {
        UiStyle {
            commands: self.entity(entity),
        }
    }
}

pub struct UiStyleUnchecked<'a> {
    commands: EntityCommands<'a>,
}

impl UiStyleUnchecked<'_> {
    pub fn id(&self) -> Entity {
        self.commands.id()
    }

    pub fn entity_commands(&mut self) -> EntityCommands {
        self.commands.reborrow()
    }
}

pub trait UiStyleUncheckedExt {
    fn style_unchecked(&mut self, entity: Entity) -> UiStyleUnchecked;
}

impl UiStyleUncheckedExt for Commands<'_, '_> {
    fn style_unchecked(&mut self, entity: Entity) -> UiStyleUnchecked {
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
