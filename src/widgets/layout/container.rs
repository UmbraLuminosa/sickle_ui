use bevy::prelude::*;

use crate::ui_builder::*;

pub trait UiContainerExt<'w, 's> {
    fn container<'a>(
        &'a mut self,
        bundle: impl Bundle,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiContainerExt<'w, 's> for UiBuilder<'w, 's, '_, UiRoot> {
    fn container<'a>(
        &'a mut self,
        bundle: impl Bundle,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut new_builder = self.spawn(bundle);
        spawn_children(&mut new_builder);

        new_builder
    }
}

impl<'w, 's> UiContainerExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn container<'a>(
        &'a mut self,
        bundle: impl Bundle,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut new_builder = self.spawn(bundle);
        spawn_children(&mut new_builder);

        new_builder
    }
}
