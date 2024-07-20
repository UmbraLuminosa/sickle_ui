use bevy::{
    ecs::{
        bundle::Bundle,
        entity::Entity,
        system::{Commands, EntityCommands},
    },
    hierarchy::BuildChildren,
    prelude::*,
};

use crate::{
    ui_commands::EntityCommandsNamedExt,
    ui_style::{UiStyle, UiStyleExt, UiStyleUnchecked, UiStyleUncheckedExt},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UiRoot;

/// Used to find a root node where nodes are safe to spawn
/// i.e. context menus or floating panels torn off from tab containers
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct UiContextRoot;

pub struct UiBuilder<'a, T> {
    commands: Commands<'a, 'a>,
    context: T,
}

impl<'a, T> UiBuilder<'a, T> {
    pub fn context(&self) -> &T {
        &self.context
    }

    pub fn commands(&mut self) -> &mut Commands<'a, 'a> {
        &mut self.commands
    }
}

impl UiBuilder<'_, UiRoot> {
    pub fn spawn(&mut self, bundle: impl Bundle) -> UiBuilder<Entity> {
        let new_entity = self.commands().spawn(bundle).id();

        self.commands().ui_builder(new_entity)
    }
}

impl UiBuilder<'_, Entity> {
    pub fn id(&self) -> Entity {
        *self.context()
    }

    pub fn entity_commands(&mut self) -> EntityCommands {
        let entity = self.id();
        self.commands().entity(entity)
    }

    /// Styling commands for UI Nodes
    ///
    /// `sickle_ui` exposes functions for all standard bevy styleable attributes.
    /// Manual extension can be done for custom styling needs via extension traits:
    ///
    /// ```rust
    /// pub trait SetMyPropExt {
    ///     fn my_prop(&mut self, value: f32) -> &mut Self;
    /// }
    ///
    /// impl SetMyPropExt for UiStyle<'_> {
    ///     fn my_prop(&mut self, value: f32) -> &mut Self {
    ///         // SetMyProp is assumed to be an EntityCommand
    ///         // Alternatively a closure can be supplied as per a standard bevy command
    ///         // NOTE: All built-in commands structs are public and can be re-used in extensions
    ///         self.entity_commands().add(SetMyProp {
    ///             value
    ///         });
    ///         self
    ///     }
    /// }
    /// ```
    pub fn style(&mut self) -> UiStyle {
        let entity = self.id();
        self.commands().style(entity)
    }

    pub fn style_unchecked(&mut self) -> UiStyleUnchecked {
        let entity = self.id();
        self.commands().style_unchecked(entity)
    }

    pub fn spawn(&mut self, bundle: impl Bundle) -> UiBuilder<Entity> {
        let mut new_entity = Entity::PLACEHOLDER;

        let entity = self.id();
        self.commands().entity(entity).with_children(|parent| {
            new_entity = parent.spawn(bundle).id();
        });

        self.commands().ui_builder(new_entity)
    }

    pub fn insert(&mut self, bundle: impl Bundle) -> &mut Self {
        self.entity_commands().insert(bundle);
        self
    }

    pub fn named(&mut self, name: impl Into<String>) -> &mut Self {
        self.entity_commands().named(name);
        self
    }
}

pub trait UiBuilderExt {
    fn ui_builder<T>(&mut self, context: T) -> UiBuilder<T>;
}

impl UiBuilderExt for Commands<'_, '_> {
    fn ui_builder<T>(&mut self, context: T) -> UiBuilder<T> {
        UiBuilder {
            commands: self.reborrow(),
            context,
        }
    }
}
