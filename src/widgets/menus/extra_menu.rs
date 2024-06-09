use bevy::prelude::*;

use sickle_ui_scaffold::ui_builder::{UiBuilder, UiBuilderExt};

use crate::widgets::layout::container::UiContainerExt;

use super::menu_bar::{MenuBar, UiMenuBarSubExt};

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ExtraMenu;

impl ExtraMenu {
    fn frame() -> impl Bundle {
        (
            Name::new("Extra Menu"),
            NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::End,
                    width: Val::Percent(100.),
                    ..default()
                },
                ..default()
            },
        )
    }
}

pub trait UiExtraMenuExt<'w> {
    fn extra_menu<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 'a, Entity>;
}

impl<'w> UiExtraMenuExt<'w> for UiBuilder<'w, '_, (Entity, MenuBar)> {
    fn extra_menu<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 'a, Entity> {
        let own_id = self.id();
        let id = self
            .commands()
            .ui_builder(own_id)
            .container((ExtraMenu::frame(), ExtraMenu), spawn_children)
            .id();

        self.commands().ui_builder(id)
    }
}
