use std::collections::VecDeque;

use bevy::prelude::*;
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    scroll_interaction::{ScrollAxis, Scrollable},
    ui_builder::UiBuilder,
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use super::{
    floating_panel::FloatingPanel,
    prelude::{
        FloatingPanelConfig, FloatingPanelLayout, LabelConfig, UiContainerExt, UiFloatingPanelExt,
        UiLabelExt,
    },
    scroll_view::ScrollThrough,
};

pub struct DropdownPlugin;

impl Plugin for DropdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_option_press,
                update_dropdown_label,
                handle_click_or_touch.after(FluxInteractionUpdate),
                update_dropdown_panel_visibility,
            )
                .chain(),
        );
    }
}

fn update_dropdown_label(
    mut q_dropdowns: Query<(&mut Dropdown, &DropdownOptions), Changed<Dropdown>>,
    mut q_text: Query<&mut Text>,
) {
    for (mut dropdown, options) in &mut q_dropdowns {
        let Ok(mut label) = q_text.get_mut(dropdown.button_label) else {
            continue;
        };

        if let Some(value) = dropdown.value {
            if value >= options.0.len() {
                dropdown.value = None;
            }
        }

        let text = if let Some(value) = dropdown.value {
            options.0[value].clone()
        } else {
            String::from("---")
        };

        label.sections = vec![TextSection::new(text, TextStyle::default())];
    }
}

fn handle_click_or_touch(
    r_mouse: Res<ButtonInput<MouseButton>>,
    r_touches: Res<Touches>,
    mut q_dropdowns: Query<(Entity, &mut Dropdown, &FluxInteraction)>,
) {
    if r_mouse.any_just_released([MouseButton::Left, MouseButton::Middle, MouseButton::Right])
        || r_touches.any_just_released()
    {
        let mut open: Option<Entity> = None;
        for (entity, _, interaction) in &mut q_dropdowns {
            if *interaction == FluxInteraction::Released {
                open = entity.into();
                break;
            }
        }

        for (entity, mut dropdown, _) in &mut q_dropdowns {
            if let Some(open_dropdown) = open {
                if entity == open_dropdown {
                    dropdown.is_open = !dropdown.is_open;
                } else if dropdown.is_open {
                    dropdown.is_open = false;
                }
            } else if dropdown.is_open {
                dropdown.is_open = false;
            }
        }
    }
}

fn handle_option_press(
    q_options: Query<(&DropdownOption, &FluxInteraction), Changed<FluxInteraction>>,
    mut q_dropdown: Query<&mut Dropdown>,
) {
    for (option, interaction) in &q_options {
        if *interaction == FluxInteraction::Released {
            let Ok(mut dropdown) = q_dropdown.get_mut(option.dropdown) else {
                continue;
            };

            dropdown.value = option.option.into();
        }
    }
}

fn update_dropdown_panel_visibility(
    mut q_panels: Query<(&DropdownPanel, &mut Visibility, &mut FloatingPanel)>,
    q_dropdown: Query<Ref<Dropdown>>,
) {
    for (panel, mut visibility, mut floating_panel) in &mut q_panels {
        let Ok(dropdown) = q_dropdown.get(panel.dropdown) else {
            continue;
        };

        if !dropdown.is_changed() {
            continue;
        }

        if dropdown.is_open {
            *visibility = Visibility::Inherited;
            floating_panel.priority = true;
        } else if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
            floating_panel.priority = false;
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct DropdownOptions(Vec<String>);

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct DropdownOption {
    dropdown: Entity,
    option: usize,
}

impl Default for DropdownOption {
    fn default() -> Self {
        Self {
            dropdown: Entity::PLACEHOLDER,
            option: Default::default(),
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct DropdownPanel {
    dropdown: Entity,
}

impl Default for DropdownPanel {
    fn default() -> Self {
        Self {
            dropdown: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Dropdown {
    value: Option<usize>,
    panel: Entity,
    button_label: Entity,
    is_open: bool,
}

impl Default for Dropdown {
    fn default() -> Self {
        Self {
            value: Default::default(),
            panel: Entity::PLACEHOLDER,
            button_label: Entity::PLACEHOLDER,
            is_open: false,
        }
    }
}

impl Dropdown {
    pub fn value(&self) -> Option<usize> {
        self.value
    }

    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn base_bundle(options: Vec<String>) -> impl Bundle {
        (
            Name::new("Dropdown"),
            ButtonBundle {
                style: Style {
                    min_width: Val::Px(150.),
                    min_height: Val::Px(26.),
                    align_self: AlignSelf::Start,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Start,
                    margin: UiRect::all(Val::Px(5.)),
                    padding: UiRect::horizontal(Val::Px(5.)),
                    ..default()
                },
                background_color: Color::GRAY.into(),
                ..default()
            },
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Color::rgba(0., 1., 1., 0.3).into(),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: Dropdown::base_tween(),
                ..default()
            },
            DropdownOptions(options),
        )
    }

    fn option_bundle(option: usize, dropdown: Entity) -> impl Bundle {
        (
            Name::new(format!("Option {}", option)),
            ButtonBundle {
                style: Style {
                    height: Val::Px(26.),
                    justify_content: JustifyContent::Start,
                    align_content: AlignContent::Center,
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
                tween: Dropdown::base_tween(),
                ..default()
            },
            DropdownOption { dropdown, option },
            ScrollThrough,
            Scrollable::default(),
        )
    }
}

pub trait UiDropdownExt<'w, 's> {
    fn dropdown<'a>(&'a mut self, options: Vec<impl Into<String>>)
        -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiDropdownExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn dropdown<'a>(
        &'a mut self,
        options: Vec<impl Into<String>>,
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut selected = Entity::PLACEHOLDER;
        let mut panel_id = Entity::PLACEHOLDER;

        let option_count = options.len();
        let mut string_options: Vec<String> = Vec::with_capacity(option_count);
        let mut queue = VecDeque::from(options);
        for _ in 0..option_count {
            let label: String = queue.pop_front().unwrap().into();
            string_options.push(label);
        }

        let mut dropdown =
            self.container(Dropdown::base_bundle(string_options.clone()), |builder| {
                let dropdown_id = builder.id();
                selected = builder
                    .label(LabelConfig {
                        margin: UiRect::right(Val::Px(10.)),
                        ..default()
                    })
                    .id();
                panel_id = builder
                    .floating_panel(
                        FloatingPanelConfig {
                            draggable: false,
                            resizable: false,
                            restrict_scroll: ScrollAxis::Vertical.into(),
                            ..default()
                        },
                        FloatingPanelLayout {
                            size: Vec2 { x: 200., y: 100. },
                            position: None,
                            hidden: true,
                            ..default()
                        },
                        |container| {
                            for (index, label) in string_options.iter().enumerate() {
                                container.container(
                                    Dropdown::option_bundle(index, dropdown_id),
                                    |option| {
                                        option.label(LabelConfig {
                                            label: label.clone(),
                                            margin: UiRect::horizontal(Val::Px(10.)),
                                            color: Color::WHITE,
                                            ..default()
                                        });
                                    },
                                );
                            }
                        },
                    )
                    .insert(DropdownPanel {
                        dropdown: dropdown_id,
                    })
                    .id();
            });

        dropdown.insert(Dropdown {
            button_label: selected,
            panel: panel_id,
            ..default()
        });

        dropdown
    }
}
