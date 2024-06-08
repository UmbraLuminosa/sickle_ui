use std::collections::VecDeque;

use bevy::{prelude::*, ui::FocusPolicy};

use sickle_math::ease::Ease;
use sickle_ui_scaffold::{
    ui_builder::UiBuilder, FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    widgets::{
        layout::{
            container::UiContainerExt,
            label::{LabelConfig, UiLabelExt},
        },
        menus::{context_menu::ContextMenuUpdate, menu::MenuUpdate, submenu::SubmenuUpdate},
    },
};

pub struct RadioGroupPlugin;

impl Plugin for RadioGroupPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            RadioGroupUpdate
                .after(FluxInteractionUpdate)
                .before(MenuUpdate)
                .before(SubmenuUpdate)
                .before(ContextMenuUpdate),
        )
        .add_systems(
            Update,
            (
                toggle_radio_button,
                update_radio_group_buttons,
                update_radio_button,
            )
                .chain()
                .in_set(RadioGroupUpdate),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct RadioGroupUpdate;

fn toggle_radio_button(
    mut q_radio_buttons: Query<(&mut RadioButton, &FluxInteraction), Changed<FluxInteraction>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q_group: Query<&mut RadioGroup>,
) {
    for (mut radio_button, interaction) in &mut q_radio_buttons {
        if *interaction == FluxInteraction::Pressed {
            let mut changed = false;

            if radio_button.checked
                && radio_button.unselectable
                && keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            {
                radio_button.checked = false;
                changed = true;
            } else if !radio_button.checked {
                radio_button.checked = true;
                changed = true;
            }

            if !changed {
                continue;
            }

            if let Some(group) = radio_button.group {
                let Ok(mut radio_group) = q_group.get_mut(group) else {
                    continue;
                };

                radio_group.selected = if radio_button.checked {
                    radio_button.index.into()
                } else {
                    None
                };
            }
        }
    }
}

fn update_radio_group_buttons(
    mut q_radio_buttons: Query<(&RadioGroup, &Children), Changed<RadioGroup>>,
    mut q_radio_button: Query<&mut RadioButton>,
) {
    for (radio_group, children) in &mut q_radio_buttons {
        for child in children {
            if let Ok(mut button) = q_radio_button.get_mut(*child) {
                // This is to avoid double triggering the change
                let checked = radio_group.selected == button.index.into();
                if button.checked != checked {
                    button.checked = checked;
                }
            }
        }
    }
}

fn update_radio_button(
    q_checkboxes: Query<&RadioButton, Changed<RadioButton>>,
    mut q_visibility: Query<&mut Visibility>,
) {
    for checkbox in &q_checkboxes {
        if let Ok(mut visiblity) = q_visibility.get_mut(checkbox.check_node) {
            *visiblity = match checkbox.checked {
                true => Visibility::Inherited,
                false => Visibility::Hidden,
            };
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct RadioGroup {
    pub selected: Option<usize>,
}

impl Default for RadioGroup {
    fn default() -> Self {
        Self { selected: None }
    }
}

impl RadioGroup {
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn select(&mut self, value: impl Into<Option<usize>>) {
        let selected = value.into();
        if self.selected != selected {
            self.selected = selected;
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct RadioButton {
    pub index: usize,
    pub checked: bool,
    unselectable: bool,
    check_node: Entity,
    group: Option<Entity>,
}

impl Default for RadioButton {
    fn default() -> Self {
        Self {
            index: 0,
            checked: false,
            unselectable: false,
            check_node: Entity::PLACEHOLDER,
            group: None,
        }
    }
}

impl RadioButton {
    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn button() -> impl Bundle {
        (
            ButtonBundle {
                style: Style {
                    height: Val::Px(26.),
                    align_self: AlignSelf::Start,
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
                tween: RadioButton::base_tween(),
                ..default()
            },
        )
    }

    fn radio_mark_background() -> impl Bundle {
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

    fn radio_mark() -> impl Bundle {
        (
            Name::new("Checkmark"),
            NodeBundle {
                style: Style {
                    width: Val::Px(10.),
                    height: Val::Px(10.),
                    margin: UiRect::all(Val::Px(2.)),
                    ..default()
                },
                background_color: Color::DARK_GRAY.into(),
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
        )
    }
}

pub trait UiRadioGroupExt<'w, 's> {
    fn radio_group<'a>(
        &'a mut self,
        options: Vec<impl Into<String>>,
        unselectable: bool,
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiRadioGroupExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn radio_group<'a>(
        &'a mut self,
        options: Vec<impl Into<String>>,
        unselectable: bool,
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let option_count = options.len();
        let mut queue = VecDeque::from(options);
        self.container(
            (
                Name::new("Radio Group"),
                NodeBundle::default(),
                RadioGroup::default(),
            ),
            |radio_group| {
                for i in 0..option_count {
                    let label = queue.pop_front().unwrap();
                    let id = radio_group.id();
                    let mut check_node: Entity = Entity::PLACEHOLDER;
                    let label_string = label.into();
                    let name = format!("Radio Button [{}]", label_string.clone());
                    radio_group
                        .container((Name::new(name), RadioButton::button()), |button| {
                            button.container(
                                RadioButton::radio_mark_background(),
                                |radio_mark_bg| {
                                    check_node =
                                        radio_mark_bg.spawn(RadioButton::radio_mark()).id();
                                },
                            );
                            button.label(LabelConfig {
                                label: label_string,
                                margin: UiRect::right(Val::Px(10.)),
                                ..default()
                            });
                        })
                        .insert(RadioButton {
                            index: i.try_into().unwrap(),
                            checked: false,
                            unselectable,
                            check_node,
                            group: id.into(),
                        });
                }
            },
        )
    }
}
