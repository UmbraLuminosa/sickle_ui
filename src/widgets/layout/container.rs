use bevy::prelude::*;

use sickle_ui_scaffold::ui_builder::{UiBuilder, UiRoot};

pub trait UiContainerExt {
    fn container(
        &mut self,
        bundle: impl Bundle,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity>;
}

impl UiContainerExt for UiBuilder<'_, UiRoot> {
    fn container(
        &mut self,
        bundle: impl Bundle,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity> {
        let mut new_builder = self.spawn(bundle);
        spawn_children(&mut new_builder);

        new_builder
    }
}

impl UiContainerExt for UiBuilder<'_, Entity> {
    fn container(
        &mut self,
        bundle: impl Bundle,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity> {
        let mut new_builder = self.spawn(bundle);
        spawn_children(&mut new_builder);

        new_builder
    }
}
