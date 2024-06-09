use bevy::{prelude::*, ui::FocusPolicy};

use sickle_ui_scaffold::prelude::*;

use crate::widgets::layout::{
    container::UiContainerExt,
    label::{LabelConfig, UiLabelExt},
};

pub struct CheckboxPlugin;

impl Plugin for CheckboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<Checkbox>::default())
            .add_systems(Update, (toggle_checkbox, update_checkbox).chain());
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

fn update_checkbox(
    q_checkboxes: Query<(Entity, &Checkbox), Changed<Checkbox>>,
    mut commands: Commands,
) {
    for (entity, checkbox) in &q_checkboxes {
        commands
            .style_unchecked(checkbox.checkmark)
            .visibility(match checkbox.checked {
                true => Visibility::Inherited,
                false => Visibility::Hidden,
            });

        match checkbox.checked {
            true => commands
                .entity(entity)
                .add_pseudo_state(PseudoState::Checked),
            false => commands
                .entity(entity)
                .remove_pseudo_state(PseudoState::Checked),
        };
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct Checkbox {
    pub checked: bool,
    checkmark_background: Entity,
    checkmark: Entity,
    label: Entity,
}

impl Default for Checkbox {
    fn default() -> Self {
        Self {
            checked: false,
            checkmark_background: Entity::PLACEHOLDER,
            checkmark: Entity::PLACEHOLDER,
            label: Entity::PLACEHOLDER,
        }
    }
}

impl UiContext for Checkbox {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            Checkbox::CHECKMARK_BACKGROUND => Ok(self.checkmark_background),
            Checkbox::CHECKMARK => Ok(self.checkmark),
            Checkbox::LABEL => Ok(self.label),
            _ => Err(format!(
                "{} doesn't exists for Checkbox. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![
            Checkbox::CHECKMARK_BACKGROUND,
            Checkbox::CHECKMARK,
            Checkbox::LABEL,
        ]
    }
}

impl DefaultTheme for Checkbox {
    fn default_theme() -> Option<Theme<Checkbox>> {
        Checkbox::theme().into()
    }
}

impl Checkbox {
    pub const CHECKMARK_BACKGROUND: &'static str = "CheckmarkBackground";
    pub const CHECKMARK: &'static str = "Checkmark";
    pub const LABEL: &'static str = "Label";

    pub fn theme() -> Theme<Checkbox> {
        let base_theme = PseudoTheme::deferred(None, Checkbox::primary_style);
        let checked_theme =
            PseudoTheme::deferred(vec![PseudoState::Checked], Checkbox::checked_style);
        Theme::new(vec![base_theme, checked_theme])
    }

    // TODO: bevy 0.14: Add border radius
    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .height(Val::Px(theme_spacing.inputs.checkbox.line_height))
            .justify_content(JustifyContent::Start)
            .align_items(AlignItems::Center)
            .margin(UiRect::horizontal(Val::Px(theme_spacing.gaps.small)))
            .background_color(Color::NONE);

        style_builder
            .switch_target(Checkbox::CHECKMARK_BACKGROUND)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .align_content(AlignContent::Center)
            .size(Val::Px(theme_spacing.inputs.checkbox.checkbox_size()))
            .margin(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .border(UiRect::all(Val::Px(
                theme_spacing.inputs.checkbox.border_size,
            )))
            .background_color(Color::NONE)
            .animated()
            .border_color(AnimatedVals {
                idle: colors.on(On::SurfaceVariant),
                hover: colors.on(On::Surface).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(Checkbox::CHECKMARK)
            .size(Val::Px(theme_spacing.inputs.checkbox.checkmark_size))
            .icon(theme_data.icons.checkmark.with(
                colors.on(On::Surface),
                theme_spacing.inputs.checkbox.checkmark_size,
            ));

        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);
        style_builder
            .switch_target(Checkbox::LABEL)
            .margin(UiRect::px(
                theme_spacing.gaps.small,
                theme_spacing.gaps.medium,
                0.,
                0.,
            ))
            .sized_font(font)
            .animated()
            .font_color(AnimatedVals {
                idle: colors.on(On::SurfaceVariant),
                hover: colors.on(On::Surface).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);
    }

    fn checked_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .switch_target(Checkbox::CHECKMARK_BACKGROUND)
            .animated()
            .border(AnimatedVals {
                idle: UiRect::all(Val::Px(0.)),
                hover: UiRect::all(Val::Px(theme_spacing.inputs.checkbox.border_size)).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(Checkbox::CHECKMARK_BACKGROUND)
            .animated()
            .background_color(AnimatedVals {
                idle: colors.container(Container::Primary),
                enter_from: Some(Color::NONE),
                ..default()
            })
            .copy_from(theme_data.enter_animation);

        style_builder
            .switch_target(Checkbox::CHECKMARK)
            .animated()
            .margin(AnimatedVals {
                idle: UiRect::all(Val::Px(theme_spacing.inputs.checkbox.border_size)),
                hover: UiRect::all(Val::Px(0.)).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(Checkbox::CHECKMARK)
            .animated()
            .scale(AnimatedVals {
                idle: 1.,
                enter_from: Some(0.),
                ..default()
            })
            .copy_from(theme_data.enter_animation);
    }

    fn checkbox_container(name: String) -> impl Bundle {
        (
            Name::new(name),
            ButtonBundle::default(),
            TrackedInteraction::default(),
        )
    }

    fn checkmark_background() -> impl Bundle {
        (
            Name::new("Checkmark Background"),
            NodeBundle {
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            LockedStyleAttributes::lock(LockableStyleAttribute::FocusPolicy),
        )
    }

    fn checkmark() -> impl Bundle {
        (
            Name::new("Checkmark"),
            ImageBundle {
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            BorderColor::default(),
            LockedStyleAttributes::from_vec(vec![
                LockableStyleAttribute::FocusPolicy,
                LockableStyleAttribute::Visibility,
            ]),
        )
    }
}

pub trait UiCheckboxExt<'w, 's> {
    fn checkbox<'a>(
        &'a mut self,
        label: Option<String>,
        checked: bool,
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiCheckboxExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn checkbox<'a>(
        &'a mut self,
        label: Option<String>,
        checked: bool,
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut checkbox = Checkbox {
            checked,
            ..default()
        };

        let label = match label {
            Some(into_string) => into_string,
            None => "".into(),
        };
        let has_label = label.len() > 0;
        let name = match has_label {
            true => format!("Checkbox [{}]", label.clone()),
            false => "Checkbox".into(),
        };

        let mut input = self.container(Checkbox::checkbox_container(name), |container| {
            checkbox.checkmark_background = container
                .container(Checkbox::checkmark_background(), |checkmark_bg| {
                    checkbox.checkmark = checkmark_bg.spawn(Checkbox::checkmark()).id();
                })
                .id();

            checkbox.label = container
                .label(LabelConfig { label, ..default() })
                .style()
                .render(has_label)
                .id();
        });

        input.insert(checkbox);

        input
    }
}
