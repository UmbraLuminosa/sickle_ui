use bevy::{input::mouse::MouseScrollUnit, ui::RelativeCursorPosition, prelude::*};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    drag_interaction::{DragState, Draggable, DraggableUpdate},
    interactions::InteractiveBackground,
    scroll_interaction::{ScrollAxis, Scrollable, ScrollableUpdate},
    ui_builder::UiBuilder,
    widgets::prelude::{LabelConfig, UiContainerExt, UiLabelExt},
    TrackedInteraction,
};

pub struct SliderPlugin;

impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_slider_on_scroll.after(ScrollableUpdate),
                update_slider_on_drag.after(DraggableUpdate),
                update_slider_handle,
                update_slider_readout,
            )
                .chain(),
        );
    }
}

// TODO: Remove hardcoded theme
// TODO: Add input for value (w/ read/write flags)
// TODO: Support click-on-bar value setting
fn update_slider_on_scroll(
    q_scrollables: Query<
        (AnyOf<(&SliderBar, &SliderDragHandle)>, &Scrollable),
        Changed<Scrollable>,
    >,
    mut q_slider: Query<&mut Slider>,
) {
    for ((slider_bar, handle), scrollable) in &q_scrollables {
        let Some((axis, diff, unit)) = scrollable.last_change() else {
            continue;
        };
        if axis == ScrollAxis::Horizontal {
            continue;
        }

        let slider_id = if let Some(slider_bar) = slider_bar {
            slider_bar.slider
        } else if let Some(handle) = handle {
            handle.slider
        } else {
            continue;
        };

        let Ok(mut slider) = q_slider.get_mut(slider_id) else {
            continue;
        };

        let offset = match unit {
            MouseScrollUnit::Line => -diff * 5.,
            MouseScrollUnit::Pixel => -diff,
        };

        let fraction = offset / 100.;
        slider.ratio = (slider.ratio + fraction).clamp(0., 1.);
    }
}

fn update_slider_on_drag(
    q_draggable: Query<(&Draggable, &SliderDragHandle, &Node), Changed<Draggable>>,
    q_node: Query<&Node>,
    mut q_slider: Query<&mut Slider>,
) {
    for (draggable, handle, node) in &q_draggable {
        let Ok(mut slider) = q_slider.get_mut(handle.slider) else {
            continue;
        };

        if draggable.state == DragState::Inactive || draggable.state == DragState::MaybeDragged {
            continue;
        }

        if draggable.state == DragState::DragCanceled {
            if let Some(base_ratio) = slider.base_ratio {
                slider.ratio = base_ratio;
                continue;
            }
        }

        if draggable.state == DragState::DragStart {
            slider.base_ratio = slider.ratio.into();
        }

        let Ok(slider_bar) = q_node.get(slider.slider_bar) else {
            continue;
        };
        let Some(diff) = draggable.diff else {
            continue;
        };

        let axis = &slider.config.axis;
        let fraction = match axis {
            SliderAxis::Horizontal => {
                let width = slider_bar.size().x - node.size().x;
                if diff.x == 0. || width == 0. {
                    continue;
                }
                diff.x / width
            }
            SliderAxis::Vertical => {
                let height = slider_bar.size().y - node.size().y;
                if diff.y == 0. || height == 0. {
                    continue;
                }
                -diff.y / height
            }
        };

        slider.ratio = (slider.ratio + fraction).clamp(0., 1.);
    }
}

fn update_slider_handle(
    q_slider: Query<&Slider, Or<(Changed<Slider>, Changed<Node>)>>,
    q_node: Query<&Node>,
    mut q_hadle_style: Query<(&Node, &mut Style), With<SliderDragHandle>>,
) {
    for slider in &q_slider {
        let Ok(slider_bar) = q_node.get(slider.slider_bar) else {
            continue;
        };
        let Ok((node, mut style)) = q_hadle_style.get_mut(slider.drag_handle) else {
            continue;
        };

        let axis = &slider.config.axis;
        match axis {
            SliderAxis::Horizontal => {
                let width = slider_bar.size().x - node.size().x;
                let handle_position = width * slider.ratio;
                if style.left != Val::Px(handle_position) {
                    style.left = Val::Px(handle_position);
                }
            }
            SliderAxis::Vertical => {
                let height = slider_bar.size().y - node.size().y;
                let handle_position = height * (1. - slider.ratio);
                if style.top != Val::Px(handle_position) {
                    style.top = Val::Px(handle_position);
                }
            }
        }
    }
}

fn update_slider_readout(
    q_slider: Query<&Slider, Changed<Slider>>,
    mut q_visibility: Query<&mut Visibility>,
    mut q_text: Query<&mut Text>,
) {
    for slider in &q_slider {
        let Some(readout_target) = slider.readout_target else {
            continue;
        };
        let Ok(mut text) = q_text.get_mut(readout_target) else {
            continue;
        };
        let Ok(mut visibility) = q_visibility.get_mut(readout_target) else {
            continue;
        };

        if slider.config.show_current {
            if *visibility == Visibility::Hidden {
                *visibility = Visibility::Inherited;
            }

            let content = format!("{:.1}", slider.value());
            let section = TextSection {
                value: content,
                style: TextStyle {
                    color: Color::ANTIQUE_WHITE,
                    font_size: 14.,
                    ..default()
                },
            };

            text.sections = vec![section];
        } else if !slider.config.show_current && *visibility == Visibility::Inherited {
            *visibility = Visibility::Hidden;
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Reflect)]
pub enum SliderAxis {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct SliderConfig {
    label: Option<String>,
    min: f32,
    max: f32,
    initial_value: f32,
    show_current: bool,
    axis: SliderAxis,
}

impl SliderConfig {
    pub fn new(
        label: Option<impl Into<String>>,
        min: f32,
        max: f32,
        initial_value: f32,
        show_current: bool,
        axis: SliderAxis,
    ) -> Self {
        if max <= min || initial_value < min || initial_value > max {
            panic!(
                "Invalid slider config values! Min: {}, Max: {}, Initial: {}",
                min, max, initial_value
            );
        }

        SliderConfig {
            min,
            max,
            initial_value,
            show_current,
            axis,
            label: SliderConfig::into_label(label),
        }
    }

    pub fn horizontal(
        label: Option<impl Into<String>>,
        min: f32,
        max: f32,
        initial_value: f32,
        show_current: bool,
    ) -> Self {
        Self::new(
            SliderConfig::into_label(label),
            min,
            max,
            initial_value,
            show_current,
            SliderAxis::Horizontal,
        )
    }

    pub fn vertical(
        label: Option<impl Into<String>>,
        min: f32,
        max: f32,
        initial_value: f32,
        show_current: bool,
    ) -> Self {
        Self::new(
            SliderConfig::into_label(label),
            min,
            max,
            initial_value,
            show_current,
            SliderAxis::Vertical,
        )
    }

    pub fn with_value(self, value: f32) -> Self {
        if value >= self.min && value <= self.max {
            return Self {
                initial_value: value,
                ..self
            };
        }

        panic!("Value must be between min and max!");
    }

    fn into_label(label: Option<impl Into<String>>) -> Option<String> {
        if let Some(label) = label {
            label.into().into()
        } else {
            None
        }
    }
}

impl Default for SliderConfig {
    fn default() -> Self {
        Self {
            label: None,
            min: 0.,
            max: 1.,
            initial_value: 0.5,
            show_current: Default::default(),
            axis: Default::default(),
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Slider {
    pub ratio: f32,
    pub config: SliderConfig,
    slider_bar: Entity,
    drag_handle: Entity,
    readout_target: Option<Entity>,
    base_ratio: Option<f32>,
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            ratio: Default::default(),
            config: Default::default(),
            slider_bar: Entity::PLACEHOLDER,
            drag_handle: Entity::PLACEHOLDER,
            readout_target: None,
            base_ratio: None,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct SliderDragHandle {
    pub slider: Entity,
}

impl Default for SliderDragHandle {
    fn default() -> Self {
        Self {
            slider: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct SliderBar {
    pub slider: Entity,
}

impl Default for SliderBar {
    fn default() -> Self {
        Self {
            slider: Entity::PLACEHOLDER,
        }
    }
}

impl Slider {
    pub fn value(&self) -> f32 {
        self.config.min.lerp(self.config.max, self.ratio)
    }

    pub fn set_value(&mut self, value: f32) {
        if value > self.config.max || value < self.config.min {
            warn!("Tried to set slider value outside of range");
            return;
        }

        self.ratio = (value - self.config.min) / (self.config.max + (0. - self.config.min))
    }

    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn horizontal_container() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Px(20.),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(5.)),
                ..default()
            },
            ..default()
        }
    }

    fn vertical_container() -> impl Bundle {
        NodeBundle {
            style: Style {
                height: Val::Percent(100.),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(5.)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        }
    }

    fn horizontal_bar_container() -> impl Bundle {
        (
            Name::new("Horizontal Bar Container"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    ..default()
                },
                ..default()
            },
            Interaction::default(),
            Scrollable::default(),
        )
    }

    fn horizontal_bar() -> impl Bundle {
        (
            Name::new("Horizontal Bar"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Px(4.),
                    margin: UiRect::vertical(Val::Px(8.)),
                    border: UiRect::px(1., 1., 0., 1.),
                    ..default()
                },
                background_color: Color::DARK_GRAY.into(),
                border_color: Color::GRAY.into(),
                ..default()
            },
        )
    }

    fn horizontal_readout_container() -> impl Bundle {
        (
            Name::new("Horizontal Readount"),
            NodeBundle {
                style: Style {
                    width: Val::Px(50.),
                    overflow: Overflow::clip(),
                    ..default()
                },
                ..default()
            },
        )
    }

    fn vertical_bar_container() -> impl Bundle {
        (
            Name::new("Vertical Bar Container"),
            NodeBundle {
                style: Style {
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            Interaction::default(),
            Scrollable::default(),
        )
    }

    fn vertical_bar() -> impl Bundle {
        (
            Name::new("Vertical Bar"),
            NodeBundle {
                style: Style {
                    width: Val::Px(4.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::horizontal(Val::Px(8.)),
                    border: UiRect::px(1., 1., 0., 1.),
                    ..default()
                },
                background_color: Color::DARK_GRAY.into(),
                border_color: Color::GRAY.into(),
                ..default()
            },
        )
    }

    fn handle_bundle(slider: Entity, axis: SliderAxis) -> impl Bundle {
        let margin = match axis {
            SliderAxis::Horizontal => UiRect::top(Val::Px(-8.)),
            SliderAxis::Vertical => UiRect::left(Val::Px(-8.)),
        };

        (
            Name::new("Handle"),
            ButtonBundle {
                style: Style {
                    width: Val::Px(20.),
                    height: Val::Px(20.),
                    border: UiRect::px(1., 1., 1., 2.),
                    margin,
                    ..default()
                },
                background_color: Color::AQUAMARINE.into(),
                border_color: Color::GRAY.into(),
                ..default()
            },
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Color::rgba(0., 1., 1., 0.8).into(),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: Slider::base_tween(),
                ..default()
            },
            SliderDragHandle { slider },
            Draggable::default(),
            RelativeCursorPosition::default(),
            Scrollable::default(),
        )
    }
}

pub trait UiSliderExt<'w, 's> {
    fn slider<'a>(&'a mut self, config: SliderConfig) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiSliderExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn slider<'a>(&'a mut self, config: SliderConfig) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut drag_handle: Entity = Entity::PLACEHOLDER;
        let mut slider_bar: Entity = Entity::PLACEHOLDER;
        let mut readout_target: Option<Entity> = None;

        let mut input = match config.axis {
            SliderAxis::Horizontal => self.container(Slider::horizontal_container(), |slider| {
                let input_id = slider.id();

                if let Some(label) = config.label.clone() {
                    let label_string: String = label.into();
                    let name_attr = format!("Horizontal Slider [{}]", label_string.clone());
                    slider.insert(Name::new(name_attr));

                    slider.label(LabelConfig {
                        label: label_string,
                        margin: UiRect::px(5., 10., 0., 0.),
                        ..default()
                    });
                }

                slider_bar = slider
                    .container(
                        (
                            Slider::horizontal_bar_container(),
                            SliderBar { slider: input_id },
                        ),
                        |bar_container| {
                            bar_container.container(Slider::horizontal_bar(), |bar| {
                                drag_handle = bar
                                    .spawn(Slider::handle_bundle(input_id, SliderAxis::Horizontal))
                                    .id();
                            });
                        },
                    )
                    .id();

                if config.show_current {
                    slider.container(
                        Slider::horizontal_readout_container(),
                        |readout_container| {
                            readout_target = readout_container
                                .label(LabelConfig {
                                    margin: UiRect::left(Val::Px(5.)),
                                    ..default()
                                })
                                .id()
                                .into();
                        },
                    );
                }
            }),
            SliderAxis::Vertical => self.container(Slider::vertical_container(), |slider| {
                let input_id = slider.id();

                if config.show_current {
                    readout_target = slider
                        .label(LabelConfig {
                            margin: UiRect::px(5., 5., 5., 0.),
                            ..default()
                        })
                        .id()
                        .into();
                }

                slider_bar = slider
                    .container(
                        (
                            Slider::vertical_bar_container(),
                            SliderBar { slider: input_id },
                        ),
                        |bar_container| {
                            bar_container.container(Slider::vertical_bar(), |bar| {
                                drag_handle = bar
                                    .spawn(Slider::handle_bundle(input_id, SliderAxis::Vertical))
                                    .id();
                            });
                        },
                    )
                    .id();

                if let Some(label) = config.label.clone() {
                    let label_string: String = label.into();
                    let name_attr = format!("Vertical Slider [{}]", label_string.clone());
                    slider.insert(Name::new(name_attr));

                    slider.label(LabelConfig {
                        label: label_string,
                        margin: UiRect::px(5., 5., 0., 5.),
                        ..default()
                    });
                }
            }),
        };

        input.insert(Slider {
            ratio: (config.initial_value - config.min) / (config.max + (0. - config.min)),
            config,
            slider_bar,
            drag_handle,
            readout_target,
            ..default()
        });

        input
    }
}
