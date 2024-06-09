use bevy::prelude::*;

use sickle_ui_scaffold::ui_builder::{UiBuilder, UiRoot};

pub trait UiContainerExt<'w> {
    fn container<'a>(
        &'a mut self,
        bundle: impl Bundle,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 'a, Entity>;
}

impl<'w> UiContainerExt<'w> for UiBuilder<'w, '_, UiRoot> {
    fn container<'a>(
        &'a mut self,
        bundle: impl Bundle,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 'a, Entity> {
        let mut new_builder = self.spawn(bundle);
        spawn_children(&mut new_builder);

        new_builder
    }
}

impl<'w> UiContainerExt<'w> for UiBuilder<'w, '_, Entity> {
    fn container<'a>(
        &'a mut self,
        bundle: impl Bundle,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 'a, Entity> {
        let mut new_builder = self.spawn(bundle);
        spawn_children(&mut new_builder);

        new_builder
    }
}
