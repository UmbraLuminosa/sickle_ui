use bevy::{prelude::*, ui::FocusPolicy};

use crate::{
    theme::{
        dynamic_style::DynamicStyle,
        theme_colors::{Accent, Container, On},
        theme_data::ThemeData,
        PseudoTheme, Theme, UiContext,
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
const CHECKMARK_BACKGROUND: &'static str = "CheckmarkBackground";
const CHECKMARK: &'static str = "Checkmark";

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
            .style(checkbox.checkmark)
            .visibility(match checkbox.checked {
                true => Visibility::Inherited,
                false => Visibility::Hidden,
            });
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct Checkbox {
    pub checked: bool,
    checkmark_background: Entity,
    checkmark: Entity,
}

impl Default for Checkbox {
    fn default() -> Self {
        Self {
            checked: false,
            checkmark_background: Entity::PLACEHOLDER,
            checkmark: Entity::PLACEHOLDER,
        }
    }
}

impl UiContext for Checkbox {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            CHECKMARK_BACKGROUND => Ok(self.checkmark_background),
            CHECKMARK => Ok(self.checkmark),
            _ => Err(format!(
                "{} doesn't exists for Checkbox. Possible contexts: {:?}",
                target,
                Checkbox::contexts()
            )),
        }
    }

    fn contexts() -> Vec<&'static str> {
        vec![CHECKMARK_BACKGROUND, CHECKMARK]
    }
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct CheckmarkBackground;

#[derive(Component, Clone, Debug, Reflect)]
pub struct Checkmark;

impl Checkbox {
    pub fn theme() -> impl Bundle {
        let container = PseudoTheme::deferred(None, Checkbox::container_style);
        Theme::<Checkbox>::new(vec![container])
    }

    fn container_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

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

        style_builder
            .context(CHECKMARK_BACKGROUND)
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

        style_builder
            .context(CHECKMARK)
            .width(Val::Px(theme_spacing.icons.extra_small))
            .height(Val::Px(theme_spacing.icons.extra_small))
            .margin(UiRect::all(Val::Px(theme_spacing.gaps.extra_small)))
            .background_color(colors.on(On::Secondary));
    }

    fn checkbox_container() -> impl Bundle {
        (ButtonBundle::default(), TrackedInteraction::default())
    }

    fn checkmark_background() -> impl Bundle {
        (
            Name::new("Checkmark Background"),
            CheckmarkBackground,
            NodeBundle {
                // TODO: Lock attribute
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
        )
    }

    fn checkmark() -> impl Bundle {
        (
            Name::new("Checkmark"),
            Checkmark,
            ImageBundle {
                // TODO: Lock attribute
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            BorderColor::default(),
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
        let mut checkmark_background: Entity = Entity::PLACEHOLDER;
        let mut checkmark: Entity = Entity::PLACEHOLDER;
        let mut name_attr: String = String::from("Checkbox");

        let mut input = self.container(Checkbox::checkbox_container(), |container| {
            checkmark_background = container
                .container(Checkbox::checkmark_background(), |checkmark_bg| {
                    let mut check_mark = checkmark_bg.spawn(Checkbox::checkmark());
                    checkmark = check_mark.id();

                    // TODO: Implement style().icon() and use theme icon
                    check_mark.style().image(CHECK_MARK);
                })
                .id();

            if let Some(label) = label {
                let label_string: String = label.into();
                name_attr = format!("Checkbox [{}]", label_string.clone());

                // TODO: Implement style().font() & theme.font(role, size class) and replace label with CheckboxLabel
                // TODO: Implement style().text_color() & .font_size() - both animatable
                // TODO: Add "role" as an optional component used for themeing via theme data
                container.label(LabelConfig {
                    label: label_string.clone(),
                    margin: UiRect::right(Val::Px(10.)),
                    ..default()
                });
            }
        });

        let checkbox = Checkbox {
            checkmark_background,
            checkmark,
            checked: value,
        };

        let theme_data = ThemeData::default();
        let mut style_builder = StyleBuilder::new();
        Checkbox::container_style(&mut style_builder, &theme_data);
        let style: DynamicStyle = style_builder.convert_with(&checkbox);

        input.insert((Name::new(name_attr), checkbox, style));

        input
    }
}
