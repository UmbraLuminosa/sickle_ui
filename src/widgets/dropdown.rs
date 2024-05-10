use std::collections::VecDeque;

use bevy::{prelude::*, ui::FocusPolicy};
use sickle_math::ease::Ease;

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
    scroll_interaction::ScrollAxis,
    theme::{
        theme_colors::{Accent, Container, On},
        theme_data::ThemeData,
        typography::{FontScale, FontStyle, FontType},
        PseudoTheme, Theme, UiContext,
    },
    ui_builder::UiBuilder,
    ui_style::{
        AnimatedVals, LockableStyleAttribute, LockedStyleAttributes, SetDisplayUncheckedExt,
        SetFocusPolicyUncheckedExt, SetHeightUncheckedExt, SetVisibilityUncheckedExt, StyleBuilder,
        UiStyleUncheckedExt,
    },
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use super::prelude::{LabelConfig, UiContainerExt, UiLabelExt, UiPanelExt, UiScrollViewExt};

const DROPDOWN_LABEL: &'static str = "Label";
const DROPDOWN_PANEL: &'static str = "Panel";
const DROPDOWN_ICON: &'static str = "Icon";
const DROPDOWN_OPTION_LABEL: &'static str = "Label";
const DROPDOWN_PANEL_Z_INDEX: usize = 11000;

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
        let Ok(mut label) = q_text.get_mut(dropdown.label) else {
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

        if label.sections.len() > 0 {
            label.sections[0].value = text;
        } else {
            label.sections = vec![TextSection::new(text, TextStyle::default())];
        }
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
    q_panels: Query<(Entity, &DropdownPanel)>,
    q_dropdown: Query<Ref<Dropdown>>,
    q_dropdown_option: Query<(&DropdownOption, &Node)>,
    q_children: Query<&Children>,
    mut commands: Commands,
) {
    for (entity, panel) in &q_panels {
        let Ok(dropdown) = q_dropdown.get(panel.dropdown) else {
            continue;
        };

        if !dropdown.is_changed() {
            continue;
        }

        if dropdown.is_open {
            let options: Vec<(&DropdownOption, &Node)> = q_children
                .iter_descendants(entity)
                .map(|option| q_dropdown_option.get(option))
                .filter(|entry| entry.is_ok())
                .map(|entry| entry.unwrap())
                .collect();
            let size = options.len() as f32 * 32.;
            // let size = options
            //     .iter()
            //     .fold(0., |acc, (_, node)| acc + node.size().y);

            commands
                .style_unchecked(entity)
                .height(Val::Px(size))
                .display(Display::Flex)
                .visibility(Visibility::Inherited);
        } else {
            commands
                .style_unchecked(entity)
                .display(Display::None)
                .visibility(Visibility::Hidden);
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
    label: Entity,
    option: usize,
}

impl Default for DropdownOption {
    fn default() -> Self {
        Self {
            dropdown: Entity::PLACEHOLDER,
            label: Entity::PLACEHOLDER,
            option: Default::default(),
        }
    }
}

impl UiContext for DropdownOption {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            DROPDOWN_OPTION_LABEL => Ok(self.label),
            _ => Err(format!(
                "{} doesn't exists for DropdownOption. Possible contexts: {:?}",
                target,
                DropdownOption::contexts()
            )),
        }
    }

    fn contexts() -> Vec<&'static str> {
        vec![DROPDOWN_OPTION_LABEL]
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

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct Dropdown {
    value: Option<usize>,
    panel: Entity,
    label: Entity,
    icon: Entity,
    is_open: bool,
}

impl Default for Dropdown {
    fn default() -> Self {
        Self {
            value: Default::default(),
            panel: Entity::PLACEHOLDER,
            label: Entity::PLACEHOLDER,
            icon: Entity::PLACEHOLDER,
            is_open: false,
        }
    }
}

impl UiContext for Dropdown {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            DROPDOWN_LABEL => Ok(self.label),
            DROPDOWN_PANEL => Ok(self.panel),
            DROPDOWN_ICON => Ok(self.icon),
            _ => Err(format!(
                "{} doesn't exists for Dropdown. Possible contexts: {:?}",
                target,
                Dropdown::contexts()
            )),
        }
    }

    fn contexts() -> Vec<&'static str> {
        vec![DROPDOWN_LABEL, DROPDOWN_PANEL, DROPDOWN_ICON]
    }
}

impl Default for Theme<Dropdown> {
    fn default() -> Self {
        Dropdown::theme()
    }
}

impl Dropdown {
    pub fn value(&self) -> Option<usize> {
        self.value
    }

    pub fn theme() -> Theme<Dropdown> {
        let base_theme = PseudoTheme::deferred(None, Dropdown::primary_style);
        Theme::<Dropdown>::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);

        style_builder
            .align_self(AlignSelf::Start)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .background_color(colors.container(Container::Primary))
            .border(UiRect::all(Val::Px(theme_spacing.borders.extra_small)))
            .min_width(Val::Px(150.))
            .min_height(Val::Px(theme_spacing.areas.medium))
            .padding(UiRect::axes(
                Val::Px(theme_spacing.gaps.medium),
                Val::Px(theme_spacing.gaps.small),
            ))
            .animated()
            .border_color(AnimatedVals {
                idle: colors.accent(Accent::Outline),
                hover: colors.on(On::Surface).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_context(DROPDOWN_LABEL)
            .sized_font(font)
            .animated()
            .font_color(AnimatedVals {
                idle: colors.on(On::PrimaryContainer),
                hover: colors.on(On::Surface).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_context(DROPDOWN_ICON)
            .width(Val::Px(theme_spacing.icons.small))
            .height(Val::Px(theme_spacing.icons.small))
            .margin(UiRect::left(Val::Px(theme_spacing.gaps.large)))
            .icon(
                theme_data
                    .icons
                    .chevron_down
                    .with(colors.on(On::PrimaryContainer), theme_spacing.icons.small),
            )
            .animated()
            .font_color(AnimatedVals {
                idle: colors.on(On::PrimaryContainer),
                hover: colors.on(On::Surface).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_context(DROPDOWN_PANEL)
            .position_type(PositionType::Absolute)
            .z_index(ZIndex::Global(DROPDOWN_PANEL_Z_INDEX as i32))
            .background_color(colors.container(Container::SurfaceHigh))
            .top(Val::Px(theme_spacing.areas.medium))
            .right(Val::Px(0.))
            .max_height(Val::Px(theme_spacing.areas.extra_large));
    }

    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn button(options: Vec<String>) -> impl Bundle {
        (
            Name::new("Dropdown"),
            ButtonBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                ..default()
            },
            TrackedInteraction::default(),
            LockedStyleAttributes::new(LockableStyleAttribute::FlexDirection),
            DropdownOptions(options),
        )
    }

    fn button_icon() -> impl Bundle {
        (
            Name::new("Dropdown Icon"),
            ImageBundle {
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            BorderColor::default(),
            LockedStyleAttributes::new(LockableStyleAttribute::FocusPolicy),
        )
    }

    fn option_bundle(option: usize) -> impl Bundle {
        (
            Name::new(format!("Option {}", option)),
            ButtonBundle {
                style: Style {
                    height: Val::Px(26.),
                    justify_content: JustifyContent::Start,
                    align_content: AlignContent::Center,
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
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
        let mut label_id = Entity::PLACEHOLDER;
        let mut panel_id = Entity::PLACEHOLDER;
        let mut icon_id = Entity::PLACEHOLDER;

        let option_count = options.len();
        let mut string_options: Vec<String> = Vec::with_capacity(option_count);
        let mut queue = VecDeque::from(options);
        for _ in 0..option_count {
            let label: String = queue.pop_front().unwrap().into();
            string_options.push(label);
        }

        let mut dropdown = self.container(Dropdown::button(string_options.clone()), |builder| {
            let dropdown_id = builder.id();
            label_id = builder.label(LabelConfig::default()).id();
            icon_id = builder.spawn(Dropdown::button_icon()).id();
            panel_id = builder
                .panel("Dropdown Options".into(), |container| {
                    container.scroll_view(ScrollAxis::Vertical, |scroll_view| {
                        for (index, label) in string_options.iter().enumerate() {
                            let mut label_id = Entity::PLACEHOLDER;
                            scroll_view.container(Dropdown::option_bundle(index), |option| {
                                label_id = option
                                    .label(LabelConfig {
                                        label: label.clone(),
                                        margin: UiRect::horizontal(Val::Px(10.)),
                                        color: Color::WHITE,
                                        ..default()
                                    })
                                    .id();

                                option.insert(DropdownOption {
                                    dropdown: dropdown_id,
                                    option: index,
                                    label: label_id,
                                });
                            });
                        }
                    });
                })
                .insert((
                    DropdownPanel {
                        dropdown: dropdown_id,
                    },
                    LockedStyleAttributes::from_vec(vec![
                        LockableStyleAttribute::Visibility,
                        LockableStyleAttribute::Display,
                        LockableStyleAttribute::FocusPolicy,
                    ]),
                ))
                .style_unchecked()
                .focus_policy(bevy::ui::FocusPolicy::Block)
                .id();
        });

        let dropdown_data = Dropdown {
            label: label_id,
            panel: panel_id,
            icon: icon_id,
            ..default()
        };

        // let style = ThemeData::with_default_and_override(
        //     Dropdown::primary_style,
        //     &dropdown_data,
        //     style_override,
        // );

        let style = ThemeData::with_default(Dropdown::primary_style).convert_with(&dropdown_data);

        dropdown.insert((dropdown_data, style));

        dropdown
    }
}
