use bevy::prelude::*;

use crate::{
    ui_builder::*,
    ui_style::{LockableStyleAttribute, LockedStyleAttributes},
};

use super::prelude::UiContainerExt;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Row;

impl Row {
    fn frame() -> impl Bundle {
        (
            Name::new("Row"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            LockedStyleAttributes::new(LockableStyleAttribute::FlexDirection),
        )
    }
}

pub trait UiRowExt<'w, 's> {
    fn row<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiRowExt<'w, 's> for UiBuilder<'w, 's, '_, UiRoot> {
    fn row<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        self.container((Row::frame(), Row), spawn_children)
    }
}

impl<'w, 's> UiRowExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn row<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        self.container((Row::frame(), Row), spawn_children)
    }
}
