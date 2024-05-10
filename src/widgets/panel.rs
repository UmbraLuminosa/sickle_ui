use bevy::prelude::*;

use crate::ui_builder::UiBuilder;

use super::prelude::UiContainerExt;

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct Panel {
    own_id: Entity,
    pub title: String,
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            own_id: Entity::PLACEHOLDER,
            title: "".into(),
        }
    }
}

impl Panel {
    pub fn own_id(&self) -> Entity {
        self.own_id
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        }
    }
}

pub trait UiPanelExt<'w, 's> {
    fn panel<'a>(
        &'a mut self,
        title: String,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiPanelExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn panel<'a>(
        &'a mut self,
        title: String,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let name = format!("Panel [{}]", title.clone());
        let mut container = self.container((Name::new(name), Panel::frame()), spawn_children);
        let own_id = container.id();

        container.insert(Panel {
            own_id,
            title,
            ..default()
        });

        container
    }
}
