use bevy::prelude::*;

use crate::widgets::floating_panel::FloatingPanel;

use super::{DynamicStyle, Theme};

pub struct DefaultThemePlugin;

impl Plugin for DefaultThemePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Theme::<FloatingPanel>::update());
    }
}

impl Default for Theme<FloatingPanel> {
    fn default() -> Self {
        Self {
            context: Default::default(),
            styles: vec![(
                None,
                DynamicStyle::build(|builder| {
                    builder.background_color(Color::BLUE);
                    builder.custom(5., |current, entity, world| {
                        let Some(mut style) = world.get_mut::<Style>(entity) else {
                            return;
                        };

                        style.border = UiRect::all(Val::Px(current));
                    });
                }),
            )],
        }
    }
}
