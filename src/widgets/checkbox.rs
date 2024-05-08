use bevy::{prelude::*, ui::FocusPolicy};

use crate::{
    theme::{
        pseudo_state::PseudoState,
        theme_colors::{Container, On},
        theme_data::ThemeData,
        typography::{FontScale, FontStyle, FontType},
        ComponentThemePlugin, PseudoTheme, Theme, UiContext,
    },
    ui_builder::UiBuilder,
    ui_commands::ManagePseudoStateExt,
    ui_style::{
        AnimatedVals, LockableStyleAttribute, LockedStyleAttributes, SetVisibilityExt,
        StyleBuilder, UiStyleExt,
    },
    FluxInteraction, TrackedInteraction,
};

use super::{
    label::LabelConfig,
    prelude::{UiContainerExt, UiLabelExt},
};

const CHECKMARK_BACKGROUND: &'static str = "CheckmarkBackground";
const CHECKMARK: &'static str = "Checkmark";
const LABEL: &'static str = "Label";

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
            .style(checkbox.checkmark)
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
            CHECKMARK_BACKGROUND => Ok(self.checkmark_background),
            CHECKMARK => Ok(self.checkmark),
            LABEL => Ok(self.label),
            _ => Err(format!(
                "{} doesn't exists for Checkbox. Possible contexts: {:?}",
                target,
                Checkbox::contexts()
            )),
        }
    }

    fn contexts() -> Vec<&'static str> {
        vec![CHECKMARK_BACKGROUND, CHECKMARK, LABEL]
    }
}

impl Default for Theme<Checkbox> {
    fn default() -> Self {
        Checkbox::theme()
    }
}

impl Checkbox {
    pub fn theme() -> Theme<Checkbox> {
        let base_theme = PseudoTheme::deferred(None, Checkbox::primary_style);
        let checked_theme =
            PseudoTheme::deferred(vec![PseudoState::Checked], Checkbox::checked_style);
        Theme::<Checkbox>::new(vec![base_theme, checked_theme])
    }

    // TODO: bevy 0.14: Add border radius
    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .height(Val::Px(theme_spacing.inputs.checkbox.line_height))
            .justify_content(JustifyContent::Start)
            .align_items(AlignItems::Center)
            .padding(UiRect::all(Val::Px(
                theme_spacing.inputs.checkbox.line_padding(),
            )))
            .background_color(Color::NONE);

        style_builder
            .switch_context(CHECKMARK_BACKGROUND)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .align_content(AlignContent::Center)
            .width(Val::Px(theme_spacing.inputs.checkbox.checkbox_size()))
            .height(Val::Px(theme_spacing.inputs.checkbox.checkbox_size()))
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
            .switch_context(CHECKMARK)
            .width(Val::Px(theme_spacing.inputs.checkbox.checkmark_size))
            .height(Val::Px(theme_spacing.inputs.checkbox.checkmark_size))
            .icon(theme_data.icons.checkmark.with(
                colors.on(On::Surface),
                theme_spacing.inputs.checkbox.checkmark_size,
            ));

        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);
        style_builder
            .switch_context(LABEL)
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
            .switch_context(CHECKMARK_BACKGROUND)
            .animated()
            .border(AnimatedVals {
                idle: UiRect::all(Val::Px(0.)),
                hover: UiRect::all(Val::Px(theme_spacing.inputs.checkbox.border_size)).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_context(CHECKMARK_BACKGROUND)
            .animated()
            .background_color(AnimatedVals {
                idle: colors.container(Container::Primary),
                enter_from: Some(Color::NONE),
                ..default()
            })
            .copy_from(theme_data.enter_animation);

        style_builder
            .switch_context(CHECKMARK)
            .animated()
            .margin(AnimatedVals {
                idle: UiRect::all(Val::Px(theme_spacing.inputs.checkbox.border_size)),
                hover: UiRect::all(Val::Px(0.)).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_context(CHECKMARK)
            .animated()
            .scale(AnimatedVals {
                idle: 1.,
                enter_from: Some(0.),
                ..default()
            })
            .copy_from(theme_data.enter_animation);
    }

    fn checkbox_container() -> impl Bundle {
        (ButtonBundle::default(), TrackedInteraction::default())
    }

    fn checkmark_background() -> impl Bundle {
        (
            Name::new("Checkmark Background"),
            NodeBundle {
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            LockedStyleAttributes::new(LockableStyleAttribute::FocusPolicy),
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
            LockedStyleAttributes::new(LockableStyleAttribute::FocusPolicy),
        )
    }
}

pub trait UiCheckboxExt<'w, 's> {
    fn checkbox<'a>(
        &'a mut self,
        label: Option<impl Into<String>>,
        value: bool,
    ) -> UiBuilder<'w, 's, 'a, Entity>;

    fn styled_checkbox<'a>(
        &'a mut self,
        label: Option<impl Into<String>>,
        value: bool,
        style_builder: impl FnOnce(&mut StyleBuilder),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiCheckboxExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn checkbox<'a>(
        &'a mut self,
        label: Option<impl Into<String>>,
        value: bool,
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        self.styled_checkbox(label, value, |_| {})
    }

    fn styled_checkbox<'a>(
        &'a mut self,
        label: Option<impl Into<String>>,
        value: bool,
        style_override: impl FnOnce(&mut StyleBuilder),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut checkmark_background: Entity = Entity::PLACEHOLDER;
        let mut checkmark: Entity = Entity::PLACEHOLDER;
        let mut label_id: Entity = Entity::PLACEHOLDER;
        let mut name_attr: String = String::from("Checkbox");

        let mut input = self.container(Checkbox::checkbox_container(), |container| {
            checkmark_background = container
                .container(Checkbox::checkmark_background(), |checkmark_bg| {
                    checkmark = checkmark_bg.spawn(Checkbox::checkmark()).id();
                })
                .id();

            if let Some(label) = label {
                let label_string: String = label.into();
                name_attr = format!("Checkbox [{}]", label_string.clone());
                label_id = container
                    .label(LabelConfig {
                        label: label_string.clone(),
                        ..default()
                    })
                    .id();
            }
        });

        let checkbox = Checkbox {
            checked: value,
            checkmark_background,
            checkmark,
            label: label_id,
        };

        let style = ThemeData::with_default_and_override(
            Checkbox::primary_style,
            &checkbox,
            style_override,
        );
        input.insert((Name::new(name_attr), checkbox, style));

        input
    }
}
