use bevy::{
    ecs::system::{EntityCommand, EntityCommands},
    prelude::*,
    ui::FocusPolicy,
};

use crate::ui_builder::*;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct LabelConfig {
    pub label: String,
    pub color: Color,
    pub margin: UiRect,
    pub wrap: FlexWrap,
    pub flex_grow: f32,
}

impl Default for LabelConfig {
    fn default() -> Self {
        Self {
            label: "Label".into(),
            color: Color::ANTIQUE_WHITE,
            margin: Default::default(),
            wrap: FlexWrap::NoWrap,
            flex_grow: 0.,
        }
    }
}

impl LabelConfig {
    pub fn from(label: impl Into<String>) -> LabelConfig {
        LabelConfig {
            label: label.into(),
            ..default()
        }
    }

    fn text_style(&self) -> TextStyle {
        TextStyle {
            color: self.color,
            font_size: 14.,
            ..default()
        }
    }

    fn frame(self) -> impl Bundle {
        let mut section = Text::from_section(self.label.clone(), self.text_style());

        if self.wrap == FlexWrap::NoWrap {
            section = section.with_no_wrap();
        }

        (
            TextBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    margin: self.margin,
                    flex_wrap: self.wrap,
                    flex_grow: self.flex_grow,
                    ..default()
                },
                text: section,
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            self,
        )
    }
}

pub trait UiLabelExt<'w, 's> {
    fn label<'a>(&'a mut self, config: LabelConfig) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiLabelExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn label<'a>(&'a mut self, config: LabelConfig) -> UiBuilder<'w, 's, 'a, Entity> {
        self.spawn((config.frame(), Label))
    }
}

struct UpdateLabelText {
    text: String,
}

impl EntityCommand for UpdateLabelText {
    fn apply(self, entity: Entity, world: &mut World) {
        let Some(config) = world.get::<LabelConfig>(entity) else {
            warn!(
                "Failed to set label text on entity {:?}: No LabelConfig component found!",
                entity
            );

            return;
        };
        let style = config.text_style();
        let Some(mut text) = world.get_mut::<Text>(entity) else {
            warn!(
                "Failed to set label text on entity {:?}: No Text component found!",
                entity
            );

            return;
        };

        text.sections = vec![TextSection::new(self.text, style)];
    }
}

pub trait SetLabelTextExt {
    fn set_label_text(&mut self, text: impl Into<String>) -> &mut Self;
}

impl SetLabelTextExt for EntityCommands<'_> {
    fn set_label_text(&mut self, text: impl Into<String>) -> &mut Self {
        self.add(UpdateLabelText { text: text.into() });

        self
    }
}
