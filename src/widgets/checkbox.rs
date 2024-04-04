use bevy::{prelude::*, ui::FocusPolicy};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    ui_builder::UiBuilder,
    ui_style::{SetImageExt, SetVisibilityExt, UiStyleExt},
    FluxInteraction, TrackedInteraction,
};

use super::{
    label::LabelConfig,
    prelude::{UiContainerExt, UiLabelExt},
};

const CHECK_MARK: &'static str = "embedded://sickle_ui/icons/checkmark.png";

pub struct CheckboxPlugin;

impl Plugin for CheckboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (toggle_checkbox, update_checkbox).chain());
    }
}

fn toggle_checkbox(
    mut q_checkboxes: Query<(&mut Checkbox, &FluxInteraction), Changed<FluxInteraction>>,
) {
    for (mut checkbox, interaction) in &mut q_checkboxes {
        if *interaction == FluxInteraction::Released {
            checkbox.checked = !checkbox.checked;
        }
    }
}

fn update_checkbox(q_checkboxes: Query<&Checkbox, Changed<Checkbox>>, mut commands: Commands) {
    for checkbox in &q_checkboxes {
        commands
            .style(checkbox.check_node)
            .visibility(match checkbox.checked {
                true => Visibility::Inherited,
                false => Visibility::Hidden,
            });
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Checkbox {
    pub checked: bool,
    check_node: Entity,
}

impl Default for Checkbox {
    fn default() -> Self {
        Self {
            checked: false,
            check_node: Entity::PLACEHOLDER,
        }
    }
}

impl Checkbox {
    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn checkbox_container() -> impl Bundle {
        (
            ButtonBundle {
                style: Style {
                    height: Val::Px(26.),
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Start,
                    margin: UiRect::all(Val::Px(5.)),
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Color::rgba(0., 1., 1., 0.3).into(),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: Checkbox::base_tween(),
                ..default()
            },
        )
    }

    fn checkmark_background() -> impl Bundle {
        (
            Name::new("Checkmark Background"),
            NodeBundle {
                style: Style {
                    width: Val::Px(16.),
                    height: Val::Px(16.),
                    margin: UiRect::all(Val::Px(5.)),
                    border: UiRect::all(Val::Px(1.)),
                    ..default()
                },
                border_color: Color::DARK_GRAY.into(),
                background_color: Color::ANTIQUE_WHITE.into(),
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
        )
    }

    fn checkmark() -> impl Bundle {
        (
            Name::new("Checkmark"),
            ImageBundle {
                style: Style {
                    width: Val::Px(10.),
                    height: Val::Px(10.),
                    margin: UiRect::all(Val::Px(2.)),
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
        )
    }
}

pub trait UiCheckboxExt<'w, 's> {
    fn checkbox<'a>(
        &'a mut self,
        label: Option<impl Into<String>>,
        value: bool,
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiCheckboxExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn checkbox<'a>(
        &'a mut self,
        label: Option<impl Into<String>>,
        value: bool,
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut check_node: Entity = Entity::PLACEHOLDER;
        let mut name_attr: String = String::from("Checkbox");

        let mut input = self.container(Checkbox::checkbox_container(), |container| {
            container.container(Checkbox::checkmark_background(), |checkmark_bg| {
                let mut check_mark = checkmark_bg.container(Checkbox::checkmark(), |_| {});
                check_node = check_mark.id();

                check_mark.style().image(CHECK_MARK);
            });

            if let Some(label) = label {
                let label_string: String = label.into();
                name_attr = format!("Checkbox [{}]", label_string.clone());

                container.label(LabelConfig {
                    label: label_string.clone(),
                    margin: UiRect::right(Val::Px(10.)),
                    ..default()
                });
            }
        });

        input.insert((
            Name::new(name_attr),
            Checkbox {
                check_node,
                checked: value,
            },
        ));

        input
    }
}
