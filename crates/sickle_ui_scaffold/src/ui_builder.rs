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

pub struct UiBuilder<'w, 's, T> {
    commands: Commands<'w, 's>,
    context: T,
}

impl<'w, 's, T> UiBuilder<'w, 's, T> {
    pub fn context(&self) -> &T {
        &self.context
    }

    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        &mut self.commands
    }
}

impl<'w> UiBuilder<'w, '_, UiRoot> {
    pub fn spawn<'a>(&'a mut self, bundle: impl Bundle) -> UiBuilder<'w, 'a, Entity> {
        let new_entity = self.commands().spawn(bundle).id();

        self.commands().ui_builder(new_entity)
    }
}

impl<'w> UiBuilder<'w, '_, Entity> {
    pub fn id(&self) -> Entity {
        *self.context()
    }

    pub fn entity_commands(&mut self) -> EntityCommands<'_> {
        let entity = self.id();
        self.commands().entity(entity)
    }

    pub fn style(&mut self) -> UiStyle<'_> {
        let entity = self.id();
        self.commands().style(entity)
    }

    pub fn style_unchecked(&mut self) -> UiStyleUnchecked<'_> {
        let entity = self.id();
        self.commands().style_unchecked(entity)
    }

    pub fn spawn<'a>(&'a mut self, bundle: impl Bundle) -> UiBuilder<'w, 'a, Entity> {
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

pub trait UiBuilderExt<'w> {
    fn ui_builder<'a,T>(&'a mut self, context: T) -> UiBuilder<'w, 'a, T>;
}

impl<'w> UiBuilderExt<'w> for Commands<'w, '_> {
    fn ui_builder<'a, T>(&'a mut self, context: T) -> UiBuilder<'w, 'a, T> {
        UiBuilder {
            commands: self.reborrow(),
            context,
        }
    }
}
