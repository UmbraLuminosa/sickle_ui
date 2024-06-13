use bevy::prelude::*;

use sickle_ui_scaffold::{
    ui_builder::{UiBuilder, UiRoot},
    ui_style::prelude::*,
};

use super::container::UiContainerExt;

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
            LockedStyleAttributes::lock(LockableStyleAttribute::FlexDirection),
        )
    }
}

pub trait UiRowExt {
    fn row(&mut self, spawn_children: impl FnOnce(&mut UiBuilder<Entity>)) -> UiBuilder<Entity>;
}

impl UiRowExt for UiBuilder<'_, UiRoot> {
    fn row(&mut self, spawn_children: impl FnOnce(&mut UiBuilder<Entity>)) -> UiBuilder<Entity> {
        self.container((Row::frame(), Row), spawn_children)
    }
}

impl UiRowExt for UiBuilder<'_, Entity> {
    fn row(&mut self, spawn_children: impl FnOnce(&mut UiBuilder<Entity>)) -> UiBuilder<Entity> {
        self.container((Row::frame(), Row), spawn_children)
    }
}
