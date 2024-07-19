use bevy::{input::mouse::MouseScrollUnit, prelude::*, ui::RelativeCursorPosition};

use sickle_ui_scaffold::{prelude::*, ui_commands::UpdateTextExt};

use crate::widgets::layout::{
    container::UiContainerExt,
    label::{LabelConfig, UiLabelExt},
};

pub struct SliderPlugin;

impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<Slider>::default())
            .add_systems(
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

        let Ok(slider_bar) = q_node.get(slider.bar_container) else {
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
        let Ok(slider_bar) = q_node.get(slider.bar_container) else {
            continue;
        };
        let Ok((node, mut style)) = q_hadle_style.get_mut(slider.handle) else {
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

fn update_slider_readout(q_slider: Query<&Slider, Changed<Slider>>, mut commands: Commands) {
    for slider in &q_slider {
        if !slider.config.show_current {
            continue;
        }

        commands
            .entity(slider.readout)
            .update_text(format!("{:.1}", slider.value()));
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Reflect)]
pub enum SliderAxis {
    #[default]
    Horizontal,
    Vertical,
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

#[derive(Component, Clone, Debug, Reflect)]
pub struct SliderConfig {
    pub label: Option<String>,
    pub min: f32,
    pub max: f32,
    pub initial_value: f32,
    pub show_current: bool,
    pub axis: SliderAxis,
}

impl SliderConfig {
    pub fn new(
        label: impl Into<Option<String>>,
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
            label: label.into(),
            min,
            max,
            initial_value,
            show_current,
            axis,
        }
    }

    pub fn horizontal(
        label: impl Into<Option<String>>,
        min: f32,
        max: f32,
        initial_value: f32,
        show_current: bool,
    ) -> Self {
        Self::new(
            label.into(),
            min,
            max,
            initial_value,
            show_current,
            SliderAxis::Horizontal,
        )
    }

    pub fn vertical(
        label: impl Into<Option<String>>,
        min: f32,
        max: f32,
        initial_value: f32,
        show_current: bool,
    ) -> Self {
        Self::new(
            label.into(),
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
    ratio: f32,
    config: SliderConfig,
    label: Entity,
    bar_container: Entity,
    bar: Entity,
    handle: Entity,
    readout_container: Entity,
    readout: Entity,
    base_ratio: Option<f32>,
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            ratio: Default::default(),
            config: Default::default(),
            base_ratio: None,
            label: Entity::PLACEHOLDER,
            bar_container: Entity::PLACEHOLDER,
            bar: Entity::PLACEHOLDER,
            handle: Entity::PLACEHOLDER,
            readout_container: Entity::PLACEHOLDER,
            readout: Entity::PLACEHOLDER,
        }
    }
}

impl UiContext for Slider {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            Slider::LABEL => Ok(self.label),
            Slider::BAR_CONTAINER => Ok(self.bar_container),
            Slider::BAR => Ok(self.bar),
            Slider::HANDLE => Ok(self.handle),
            Slider::READOUT_CONTAINER => Ok(self.readout_container),
            Slider::READOUT => Ok(self.readout),
            _ => Err(format!(
                "{} doesn't exists for Slider. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![
            Slider::LABEL,
            Slider::BAR_CONTAINER,
            Slider::BAR,
            Slider::HANDLE,
            Slider::READOUT_CONTAINER,
            Slider::READOUT,
        ]
    }
}

impl DefaultTheme for Slider {
    fn default_theme() -> Option<Theme<Slider>> {
        Slider::theme().into()
    }
}

impl Slider {
    pub const LABEL: &'static str = "Label";
    pub const BAR_CONTAINER: &'static str = "BarContainer";
    pub const BAR: &'static str = "Bar";
    pub const HANDLE: &'static str = "Handle";
    pub const READOUT_CONTAINER: &'static str = "ReadoutContainer";
    pub const READOUT: &'static str = "Readout";

    pub fn value(&self) -> f32 {
        self.config.min.lerp(self.config.max, self.ratio)
    }

    pub fn config(&self) -> &SliderConfig {
        &self.config
    }

    pub fn set_value(&mut self, value: f32) {
        if value > self.config.max || value < self.config.min {
            warn!("Tried to set slider value outside of range");
            return;
        }

        self.ratio = (value - self.config.min) / (self.config.max + (0. - self.config.min))
    }

    pub fn theme() -> Theme<Slider> {
        let base_theme = PseudoTheme::deferred_context(None, Slider::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, slider: &Slider, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);

        match slider.config().axis {
            SliderAxis::Horizontal => {
                style_builder
                    .justify_content(JustifyContent::SpaceBetween)
                    .align_items(AlignItems::Center)
                    .width(Val::Percent(100.))
                    .height(Val::Px(theme_spacing.areas.small))
                    .padding(UiRect::horizontal(Val::Px(theme_spacing.gaps.medium)));

                style_builder
                    .switch_target(Slider::LABEL)
                    .margin(UiRect::right(Val::Px(theme_spacing.gaps.medium)));

                style_builder
                    .switch_target(Slider::BAR_CONTAINER)
                    .width(Val::Percent(100.));

                style_builder
                    .switch_target(Slider::BAR)
                    .width(Val::Percent(100.))
                    .height(Val::Px(theme_spacing.gaps.small))
                    .margin(UiRect::vertical(Val::Px(theme_spacing.gaps.medium)));

                style_builder
                    .switch_target(Slider::READOUT)
                    .min_width(Val::Px(theme_spacing.areas.medium))
                    .margin(UiRect::left(Val::Px(theme_spacing.gaps.medium)));

                style_builder
                    .switch_context(Slider::HANDLE, None)
                    .margin(UiRect::top(Val::Px(
                        -theme_spacing.gaps.medium + theme_spacing.borders.extra_small,
                    )));
            }
            SliderAxis::Vertical => {
                style_builder
                    .flex_direction(FlexDirection::ColumnReverse)
                    .justify_content(JustifyContent::SpaceBetween)
                    .align_items(AlignItems::Center)
                    .height(Val::Percent(100.))
                    .padding(UiRect::vertical(Val::Px(theme_spacing.gaps.medium)));

                style_builder
                    .switch_target(Slider::LABEL)
                    .margin(UiRect::px(
                        theme_spacing.gaps.small,
                        theme_spacing.gaps.small,
                        theme_spacing.gaps.small,
                        0.,
                    ));

                style_builder
                    .switch_target(Slider::BAR_CONTAINER)
                    .flex_direction(FlexDirection::Column)
                    .height(Val::Percent(100.));

                style_builder
                    .switch_target(Slider::BAR)
                    .flex_direction(FlexDirection::Column)
                    .width(Val::Px(theme_spacing.gaps.small))
                    .height(Val::Percent(100.))
                    .margin(UiRect::horizontal(Val::Px(theme_spacing.gaps.medium)));

                style_builder
                    .switch_target(Slider::READOUT_CONTAINER)
                    .justify_content(JustifyContent::Center)
                    .justify_items(JustifyItems::Center)
                    .width(Val::Px(theme_spacing.areas.medium))
                    .overflow(Overflow::clip());

                style_builder
                    .switch_target(Slider::READOUT)
                    .margin(UiRect::all(Val::Px(theme_spacing.gaps.small)));

                style_builder
                    .switch_context(Slider::HANDLE, None)
                    .margin(UiRect::left(Val::Px(
                        -theme_spacing.gaps.medium + theme_spacing.borders.extra_small,
                    )));
            }
        }

        style_builder.reset_context();

        style_builder
            .switch_target(Slider::LABEL)
            .sized_font(font.clone())
            .font_color(colors.on(On::Surface));

        if slider.config().label.is_none() {
            style_builder
                .switch_target(Slider::LABEL)
                .display(Display::None)
                .visibility(Visibility::Hidden);
        } else {
            style_builder
                .switch_target(Slider::LABEL)
                .display(Display::Flex)
                .visibility(Visibility::Inherited);
        }

        if !slider.config().show_current {
            style_builder
                .switch_target(Slider::READOUT_CONTAINER)
                .display(Display::None)
                .visibility(Visibility::Hidden);
        } else {
            style_builder
                .switch_target(Slider::READOUT_CONTAINER)
                .display(Display::Flex)
                .visibility(Visibility::Inherited);
        }

        style_builder
            .switch_target(Slider::READOUT)
            .sized_font(font.clone())
            .font_color(colors.on(On::Surface));

        style_builder
            .switch_target(Slider::BAR)
            .border(UiRect::px(
                0.,
                theme_spacing.borders.extra_small,
                0.,
                theme_spacing.borders.extra_small,
            ))
            .background_color(colors.surface(Surface::SurfaceVariant))
            .border_color(colors.accent(Accent::Shadow));

        style_builder
            .switch_context(Slider::HANDLE, None)
            .size(Val::Px(theme_spacing.icons.small))
            .border(UiRect::all(Val::Px(theme_spacing.borders.extra_small)))
            .border_color(colors.accent(Accent::Shadow))
            .border_radius(BorderRadius::all(Val::Px(theme_spacing.icons.small)))
            .animated()
            .background_color(AnimatedVals {
                idle: colors.accent(Accent::Primary),
                hover: colors.container(Container::Primary).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);
    }

    fn container(name: String) -> impl Bundle {
        (Name::new(name), NodeBundle::default())
    }

    fn bar_container() -> impl Bundle {
        (
            Name::new("Bar Container"),
            NodeBundle::default(),
            Interaction::default(),
            Scrollable::default(),
        )
    }

    fn bar() -> impl Bundle {
        (Name::new("Slider Bar"), NodeBundle::default())
    }

    fn handle(slider: Entity) -> impl Bundle {
        (
            Name::new("Handle"),
            ButtonBundle::default(),
            TrackedInteraction::default(),
            SliderDragHandle { slider },
            Draggable::default(),
            RelativeCursorPosition::default(),
            Scrollable::default(),
        )
    }

    fn readout_container() -> impl Bundle {
        (Name::new("Readout"), NodeBundle::default())
    }
}

pub trait UiSliderExt {
    fn slider(&mut self, config: SliderConfig) -> UiBuilder<Entity>;
}

impl UiSliderExt for UiBuilder<'_, Entity> {
    fn slider(&mut self, config: SliderConfig) -> UiBuilder<Entity> {
        let mut slider = Slider {
            ratio: (config.initial_value - config.min) / (config.max + (0. - config.min)),
            config: config.clone(),
            ..default()
        };

        let label = match config.label {
            Some(label) => label,
            None => "".into(),
        };
        let has_label = label.len() > 0;
        let name = match has_label {
            true => format!("Slider [{}]", label.clone()),
            false => "Slider".into(),
        };

        let mut input = self.container(Slider::container(name), |container| {
            let input_id = container.id();

            slider.label = container.label(LabelConfig { label, ..default() }).id();
            slider.bar_container = container
                .container(
                    (Slider::bar_container(), SliderBar { slider: input_id }),
                    |bar_container| {
                        slider.bar = bar_container
                            .container(Slider::bar(), |bar| {
                                slider.handle = bar.spawn(Slider::handle(input_id)).id();
                            })
                            .id();
                    },
                )
                .id();

            slider.readout_container = container
                .container(Slider::readout_container(), |readout_container| {
                    slider.readout = readout_container.label(LabelConfig::default()).id();
                })
                .id();
        });

        input.insert(slider);

        input
    }
}
