use bevy::prelude::*;

use sickle_ui_scaffold::{
    ui_builder::*,
    ui_style::prelude::*,
};

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Icon;

impl Icon {
    fn bundle() -> impl Bundle {
        ImageBundle {
            style: Style {
                width: Val::Px(16.),
                height: Val::Px(16.),
                ..default()
            },
            ..default()
        }
    }
}

pub trait UiIconExt<'w, 's> {
    fn icon<'a>(&'a mut self, path: impl Into<String>) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiIconExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn icon<'a>(&'a mut self, path: impl Into<String>) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut icon = self.spawn((Name::new("Icon"), Icon::bundle(), Icon));

        icon.style().image(ImageSource::Path(path.into()));

        icon
    }
}
