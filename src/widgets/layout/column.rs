use bevy::prelude::*;

use sickle_ui_scaffold::{
    ui_builder::{UiBuilder, UiRoot},
    ui_style::prelude::*,
};

use super::container::UiContainerExt;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Column;

impl Column {
    fn frame() -> impl Bundle {
        (
            Name::new("Column"),
            NodeBundle {
                style: Style {
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            LockedStyleAttributes::lock(LockableStyleAttribute::FlexDirection),
        )
    }
}

pub trait UiColumnExt<'w> {
    fn column<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 'a, Entity>;
}

impl<'w> UiColumnExt<'w> for UiBuilder<'w, '_, UiRoot> {
    fn column<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 'a, Entity> {
        self.container((Column::frame(), Column), spawn_children)
    }
}

impl<'w> UiColumnExt<'w> for UiBuilder<'w, '_, Entity> {
    fn column<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 'a, Entity> {
        self.container((Column::frame(), Column), spawn_children)
    }
}
