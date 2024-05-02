use bevy::{prelude::*, ui::FocusPolicy};

use crate::{
    theme::{
        dynamic_style::DynamicStyle,
        theme_colors::{Accent, Container, On},
        theme_data::ThemeData,
        PseudoTheme, Theme,
    },
    ui_builder::UiBuilder,
    ui_style::{AnimatedVals, SetImageExt, SetVisibilityExt, StyleBuilder, UiStyleExt},
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

#[derive(Component, Debug, Reflect)]
pub struct CheckmarkBackground;

#[derive(Component, Debug, Reflect)]
pub struct Checkmark;

#[derive(Bundle, Debug)]
pub struct CheckboxTheme {
    container: Theme<Checkbox>,
    checkmark_bg: Theme<CheckmarkBackground>,
    checkmark: Theme<Checkmark>,
}

impl Checkbox {
    pub fn theme() -> impl Bundle {
        let container = PseudoTheme::deferred(None, Checkbox::container_style);
        let checkmark_bg = PseudoTheme::deferred(None, Checkbox::checkmark_background_style);
        let checkmark = PseudoTheme::deferred(None, Checkbox::checkmark_style);

        CheckboxTheme {
            container: Theme::<Checkbox>::new(vec![container]),
            checkmark_bg: Theme::<CheckmarkBackground>::new(vec![checkmark_bg]),
            checkmark: Theme::<Checkmark>::new(vec![checkmark]),
        }
    }

    fn container_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        style_builder
            .height(Val::Px(theme_spacing.areas.small))
            .align_content(AlignContent::Center)
            .justify_content(JustifyContent::Start)
            .margin(UiRect::all(Val::Px(theme_spacing.gaps.small)));

        style_builder
            .animated()
            .background_color(AnimatedVals {
                idle: Color::NONE,
                hover: theme_data.colors().accent(Accent::Secondary).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);
    }

    fn checkbox_container() -> impl Bundle {
        let theme_data = ThemeData::default();
        let mut style_builder = StyleBuilder::new();

        Checkbox::container_style(&mut style_builder, &theme_data);
        let style: DynamicStyle = style_builder.into();

        (
            ButtonBundle { ..default() },
            TrackedInteraction::default(),
            style,
        )
    }

    fn checkmark_background_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .width(Val::Px(theme_spacing.areas.extra_small))
            .height(Val::Px(theme_spacing.areas.extra_small))
            .margin(UiRect::px(
                theme_spacing.gaps.small,
                theme_spacing.gaps.medium,
                theme_spacing.gaps.small,
                theme_spacing.gaps.small,
            ))
            .border(UiRect::all(Val::Px(theme_spacing.borders.extra_small)))
            .border_color(colors.accent(Accent::Outline))
            .background_color(colors.container(Container::Secondary));
    }

    fn checkmark_background() -> impl Bundle {
        let theme_data = ThemeData::default();
        let mut style_builder = StyleBuilder::new();

        Checkbox::checkmark_background_style(&mut style_builder, &theme_data);
        let style: DynamicStyle = style_builder.into();

        (
            Name::new("Checkmark Background"),
            CheckmarkBackground,
            NodeBundle {
                // TODO: Lock attribute
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            style,
        )
    }

    fn checkmark_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .width(Val::Px(theme_spacing.icons.extra_small))
            .height(Val::Px(theme_spacing.icons.extra_small))
            .margin(UiRect::all(Val::Px(theme_spacing.gaps.extra_small)))
            .background_color(colors.on(On::Secondary));
    }

    fn checkmark() -> impl Bundle {
        let theme_data = ThemeData::default();
        let mut style_builder = StyleBuilder::new();

        Checkbox::checkmark_style(&mut style_builder, &theme_data);
        let style: DynamicStyle = style_builder.into();

        (
            Name::new("Checkmark"),
            Checkmark,
            ImageBundle {
                // TODO: Lock attribute
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            style,
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
                let mut check_mark = checkmark_bg.spawn(Checkbox::checkmark());
                check_node = check_mark.id();

                // TODO: Implement style().icon() and use theme icon
                check_mark.style().image(CHECK_MARK);
            });

            if let Some(label) = label {
                let label_string: String = label.into();
                name_attr = format!("Checkbox [{}]", label_string.clone());

                // TODO: Implement style().font() & theme.font(role, size class) and replace label with CheckboxLabel
                // TODO: Implement style().text_color() & .font_size() - both animatable
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
