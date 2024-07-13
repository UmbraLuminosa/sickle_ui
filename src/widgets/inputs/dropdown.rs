use std::collections::VecDeque;

use bevy::{prelude::*, ui::FocusPolicy};

use sickle_ui_scaffold::{prelude::*, ui_commands::UpdateTextExt};

use crate::widgets::layout::{
    container::UiContainerExt,
    label::{LabelConfig, UiLabelExt},
    panel::UiPanelExt,
    scroll_view::{ScrollView, ScrollViewLayoutUpdate, UiScrollViewExt},
};

const DROPDOWN_PANEL_Z_INDEX: usize = 11000;

pub struct DropdownPlugin;

impl Plugin for DropdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ComponentThemePlugin::<Dropdown>::default(),
            ComponentThemePlugin::<DropdownOption>::default(),
        ))
        .add_systems(
            Update,
            (
                handle_option_press,
                update_dropdown_label,
                handle_click_or_touch.after(FluxInteractionUpdate),
                update_drowdown_pseudo_state,
                update_dropdown_panel_visibility.before(ScrollViewLayoutUpdate),
            )
                .chain(),
        );
    }
}

fn update_dropdown_label(
    mut q_dropdowns: Query<(&mut Dropdown, &DropdownOptions), Changed<Dropdown>>,
    mut commands: Commands,
) {
    for (mut dropdown, options) in &mut q_dropdowns {
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

        commands.entity(dropdown.label).update_text(text);
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

fn update_drowdown_pseudo_state(
    q_panels: Query<(&DropdownPanel, &PseudoStates), Changed<PseudoStates>>,
    mut commands: Commands,
) {
    for (panel, states) in &q_panels {
        if states.has(&PseudoState::Visible) {
            commands
                .entity(panel.dropdown)
                .add_pseudo_state(PseudoState::Open);
        } else {
            commands
                .entity(panel.dropdown)
                .remove_pseudo_state(PseudoState::Open);
        }
    }
}

fn update_dropdown_panel_visibility(
    q_dropdowns: Query<&Dropdown, Changed<Dropdown>>,
    mut q_scroll_view: Query<&mut ScrollView>,
    mut commands: Commands,
) {
    for dropdown in &q_dropdowns {
        if dropdown.is_open {
            commands
                .style_unchecked(dropdown.panel)
                .display(Display::Flex)
                .visibility(Visibility::Inherited)
                .height(Val::Px(0.));

            let Ok(mut scroll_view) = q_scroll_view.get_mut(dropdown.scroll_view) else {
                continue;
            };

            scroll_view.disabled = true;
        } else {
            commands
                .style_unchecked(dropdown.panel)
                .display(Display::None)
                .visibility(Visibility::Hidden);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DropdownPanelAnchor {
    TopLeft,
    TopRight,
    #[default]
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DropdownPanelPlacement {
    pub anchor: DropdownPanelAnchor,
    pub top: Val,
    pub right: Val,
    pub bottom: Val,
    pub left: Val,
    pub width: Val,
    pub height: Val,
    pub panel_width: f32,
    pub button_width: f32,
    pub wider_than_button: bool,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct DropdownOptions(Vec<String>);

impl DropdownOptions {
    pub fn labels(&self) -> &Vec<String> {
        &self.0
    }
}

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
            DropdownOption::LABEL => Ok(self.label),
            _ => Err(format!(
                "{} doesn't exists for DropdownOption. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![DropdownOption::LABEL]
    }
}

impl DefaultTheme for DropdownOption {
    fn default_theme() -> Option<Theme<DropdownOption>> {
        DropdownOption::theme().into()
    }
}

impl DropdownOption {
    pub const LABEL: &'static str = "Label";

    pub fn dropdown(&self) -> Entity {
        self.dropdown
    }

    pub fn option(&self) -> usize {
        self.option
    }

    pub fn theme() -> Theme<DropdownOption> {
        let base_theme = PseudoTheme::deferred(None, DropdownOption::primary_style);

        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);

        style_builder
            .align_items(AlignItems::Center)
            .min_width(Val::Percent(100.))
            .padding(UiRect::axes(
                Val::Px(theme_spacing.gaps.medium),
                Val::Px(theme_spacing.gaps.medium),
            ))
            .margin(UiRect::bottom(Val::Px(theme_spacing.gaps.tiny)))
            .animated()
            .background_color(AnimatedVals {
                idle: colors.container(Container::Primary),
                hover: colors.accent(Accent::Primary).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(DropdownOption::LABEL)
            .sized_font(font)
            .animated()
            .font_color(AnimatedVals {
                idle: colors.on(On::PrimaryContainer),
                hover: colors.on(On::Primary).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);
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
    label: Entity,
    icon: Entity,
    panel: Entity,
    scroll_view: Entity,
    scroll_view_content: Entity,
    is_open: bool,
}

impl Default for Dropdown {
    fn default() -> Self {
        Self {
            value: Default::default(),
            label: Entity::PLACEHOLDER,
            icon: Entity::PLACEHOLDER,
            panel: Entity::PLACEHOLDER,
            scroll_view: Entity::PLACEHOLDER,
            scroll_view_content: Entity::PLACEHOLDER,
            is_open: false,
        }
    }
}

impl UiContext for Dropdown {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            Dropdown::LABEL => Ok(self.label),
            Dropdown::ICON => Ok(self.icon),
            Dropdown::PANEL => Ok(self.panel),
            Dropdown::SCROLL_VIEW => Ok(self.scroll_view),
            Dropdown::SCROLL_VIEW_CONTENT => Ok(self.scroll_view_content),
            _ => Err(format!(
                "{} doesn't exists for Dropdown. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![
            Dropdown::LABEL,
            Dropdown::ICON,
            Dropdown::PANEL,
            Dropdown::SCROLL_VIEW,
            Dropdown::SCROLL_VIEW_CONTENT,
        ]
    }
}

impl DefaultTheme for Dropdown {
    fn default_theme() -> Option<Theme<Dropdown>> {
        Dropdown::theme().into()
    }
}

impl Dropdown {
    pub const LABEL: &'static str = "Label";
    pub const ICON: &'static str = "Icon";
    pub const PANEL: &'static str = "Panel";
    pub const SCROLL_VIEW: &'static str = "ScrollView";
    pub const SCROLL_VIEW_CONTENT: &'static str = "ScrollViewContent";

    pub fn value(&self) -> Option<usize> {
        self.value
    }

    pub fn set_value(&mut self, value: impl Into<Option<usize>>) {
        let value = value.into();
        if self.value != value {
            self.value = value;
        }
    }

    pub fn options_container(&self) -> Entity {
        self.scroll_view_content
    }

    pub fn theme() -> Theme<Dropdown> {
        let base_theme = PseudoTheme::deferred(None, Dropdown::primary_style);
        let open_theme = PseudoTheme::deferred_world(vec![PseudoState::Open], Dropdown::open_style);

        Theme::new(vec![base_theme, open_theme])
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
            .height(Val::Px(theme_spacing.areas.small))
            .padding(UiRect::axes(
                Val::Px(theme_spacing.gaps.medium),
                Val::Px(theme_spacing.gaps.extra_small),
            ))
            .border(UiRect::all(Val::Px(0.)))
            .border_color(colors.accent(Accent::Outline))
            .border_radius(BorderRadius::all(Val::Px(
                theme_spacing.corners.extra_small,
            )))
            .animated()
            .background_color(AnimatedVals {
                idle: colors.accent(Accent::Primary),
                hover: colors.container(Container::Primary).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(Dropdown::LABEL)
            .sized_font(font)
            .animated()
            .font_color(AnimatedVals {
                idle: colors.on(On::Primary),
                hover: colors.on(On::PrimaryContainer).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(Dropdown::ICON)
            .size(Val::Px(theme_spacing.icons.small))
            .margin(UiRect::left(Val::Px(theme_spacing.gaps.large)))
            .icon(
                theme_data
                    .icons
                    .expand_more
                    .with(colors.on(On::Primary), theme_spacing.icons.small),
            )
            .animated()
            .font_color(AnimatedVals {
                idle: colors.on(On::Primary),
                hover: colors.on(On::PrimaryContainer).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(Dropdown::PANEL)
            .position_type(PositionType::Absolute)
            .min_width(Val::Percent(100.))
            .max_height(Val::Px(theme_spacing.areas.extra_large))
            .top(Val::Px(theme_spacing.areas.medium))
            .z_index(ZIndex::Global(DROPDOWN_PANEL_Z_INDEX as i32))
            .border(UiRect::all(Val::Px(theme_spacing.gaps.tiny)))
            .border_color(colors.accent(Accent::Shadow))
            .background_color(colors.container(Container::Primary));

        style_builder
            .switch_target(Dropdown::SCROLL_VIEW_CONTENT)
            .margin(UiRect::px(
                0.,
                theme_spacing.scroll_bar_size,
                0.,
                theme_spacing.scroll_bar_size,
            ));
    }

    fn open_style(style_builder: &mut StyleBuilder, entity: Entity, _: &Dropdown, world: &World) {
        let placement = match Dropdown::panel_placement_for(entity, world) {
            Ok(placement) => placement,
            Err(msg) => {
                error!("Error placing Dropdown panel: {}", msg);
                return;
            }
        };

        let theme_data = world.resource::<ThemeData>();
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let enter_animation = theme_data.enter_animation.clone();
        let corner_from_width =
            placement.panel_width < placement.button_width - theme_spacing.corners.extra_small;
        let extra_small_border = theme_spacing.borders.extra_small;
        let extra_small_corner = theme_spacing.corners.extra_small;
        let maybe_button_corner = match corner_from_width {
            true => extra_small_corner,
            false => 0.,
        };

        style_builder.background_color(colors.container(Container::Primary));

        match placement.anchor {
            DropdownPanelAnchor::TopLeft => {
                style_builder
                    .border(UiRect::top(Val::Px(extra_small_border)))
                    .border_radius(BorderRadius::px(
                        0.,
                        maybe_button_corner,
                        extra_small_corner,
                        extra_small_corner,
                    ));
            }
            DropdownPanelAnchor::TopRight => {
                style_builder
                    .border(UiRect::top(Val::Px(extra_small_border)))
                    .border_radius(BorderRadius::px(
                        maybe_button_corner,
                        0.,
                        extra_small_corner,
                        extra_small_corner,
                    ));
            }
            DropdownPanelAnchor::BottomLeft => {
                style_builder
                    .border(UiRect::bottom(Val::Px(extra_small_border)))
                    .border_radius(BorderRadius::px(
                        extra_small_corner,
                        extra_small_corner,
                        maybe_button_corner,
                        0.,
                    ));
            }
            DropdownPanelAnchor::BottomRight => {
                style_builder
                    .border(UiRect::bottom(Val::Px(extra_small_border)))
                    .border_radius(BorderRadius::px(
                        extra_small_corner,
                        extra_small_corner,
                        0.,
                        maybe_button_corner,
                    ));
            }
        }

        style_builder
            .switch_target(Dropdown::LABEL)
            .font_color(colors.on(On::PrimaryContainer));
        style_builder
            .switch_target(Dropdown::ICON)
            .font_color(colors.on(On::PrimaryContainer));

        style_builder
            .switch_target(Dropdown::PANEL)
            .top(placement.top)
            .right(placement.right)
            .bottom(placement.bottom)
            .left(placement.left)
            .width(placement.width)
            .animated()
            .height(AnimatedVals {
                idle: placement.height,
                enter_from: Val::Px(0.).into(),
                ..default()
            })
            .copy_from(enter_animation);

        let maybe_border = match placement.wider_than_button {
            true => extra_small_border,
            false => 0.,
        };

        let maybe_corner = match placement.wider_than_button {
            true => extra_small_corner,
            false => 0.,
        };

        match placement.anchor {
            DropdownPanelAnchor::TopLeft => {
                style_builder
                    .switch_target(Dropdown::PANEL)
                    .border(UiRect::px(0., maybe_border, extra_small_border, 0.))
                    .border_radius(BorderRadius::px(
                        extra_small_corner,
                        extra_small_corner,
                        maybe_corner,
                        0.,
                    ));
            }
            DropdownPanelAnchor::TopRight => {
                style_builder
                    .switch_target(Dropdown::PANEL)
                    .border(UiRect::px(maybe_border, 0., extra_small_border, 0.))
                    .border_radius(BorderRadius::px(
                        extra_small_corner,
                        extra_small_corner,
                        0.,
                        maybe_corner,
                    ));
            }
            DropdownPanelAnchor::BottomLeft => {
                style_builder
                    .switch_target(Dropdown::PANEL)
                    .border(UiRect::px(0., maybe_border, 0., extra_small_border))
                    .border_radius(BorderRadius::px(
                        0.,
                        maybe_corner,
                        extra_small_corner,
                        extra_small_corner,
                    ));
            }
            DropdownPanelAnchor::BottomRight => {
                style_builder
                    .switch_target(Dropdown::PANEL)
                    .border(UiRect::px(maybe_border, 0., 0., extra_small_border))
                    .border_radius(BorderRadius::px(
                        maybe_corner,
                        0.,
                        extra_small_corner,
                        extra_small_corner,
                    ));
            }
        }

        style_builder
            .switch_target(Dropdown::SCROLL_VIEW)
            .animated()
            .tracked_style_state(TrackedStyleState::default_vals())
            .copy_from(enter_animation);
    }

    pub fn panel_placement_for(
        entity: Entity,
        world: &World,
    ) -> Result<DropdownPanelPlacement, String> {
        let Some(dropdown) = world.get::<Dropdown>(entity) else {
            return Err("Entity has no Dropdown component".into());
        };
        let dropdown_panel = dropdown.panel;
        let scroll_view_content = dropdown.scroll_view_content;

        // Unsafe unwrap: If a UI element doesn't have a Node, we should panic!
        let dropdown_node = world.get::<Node>(entity).unwrap();
        let dropdown_size = dropdown_node.unrounded_size();
        let dropdown_borders = UiUtils::border_as_px(entity, world);
        let panel_borders = UiUtils::border_as_px(dropdown_panel, world);

        let (container_size, dropdown_position) = UiUtils::container_size_and_offset(entity, world);
        let tl_corner = dropdown_position - dropdown_size / 2.;
        let halfway_point = container_size / 2.;
        let anchor = if tl_corner.x > halfway_point.x {
            if tl_corner.y > halfway_point.y {
                DropdownPanelAnchor::TopRight
            } else {
                DropdownPanelAnchor::BottomRight
            }
        } else {
            if tl_corner.y > halfway_point.y {
                DropdownPanelAnchor::TopLeft
            } else {
                DropdownPanelAnchor::BottomLeft
            }
        };

        let panel_size_limit = match anchor {
            DropdownPanelAnchor::TopLeft => Vec2::new(container_size.x - tl_corner.x, tl_corner.y),
            DropdownPanelAnchor::TopRight => Vec2::new(tl_corner.x + dropdown_size.x, tl_corner.y),
            DropdownPanelAnchor::BottomLeft => Vec2::new(
                container_size.x - tl_corner.x,
                container_size.y - (tl_corner.y + dropdown_size.y),
            ),
            DropdownPanelAnchor::BottomRight => Vec2::new(
                tl_corner.x + dropdown_size.x,
                container_size.y - (tl_corner.y + dropdown_size.y),
            ),
        }
        .max(Vec2::ZERO);

        let Some(option_list) = world.get::<Children>(scroll_view_content) else {
            return Err("Dropdown has no options".into());
        };

        let option_list: Vec<Entity> = option_list.iter().map(|child| *child).collect();
        let mut five_children_height = panel_borders.x + panel_borders.z;
        let mut counted = 0;
        for child in option_list {
            let Some(option_node) = world.get::<Node>(child) else {
                continue;
            };

            if counted < 5 {
                five_children_height += option_node.unrounded_size().y;

                let margin_sizes = UiUtils::margin_as_px(child, world);
                five_children_height += margin_sizes.x + margin_sizes.z;
                counted += 1;
            }
        }

        // Unsafe unwrap: If a ScrollView's content doesn't have a Node, we should panic!
        let panel_width = (world
            .get::<Node>(scroll_view_content)
            .unwrap()
            .unrounded_size()
            .x
            + panel_borders.y
            + panel_borders.w)
            .clamp(0., panel_size_limit.x.max(0.));
        let idle_height = five_children_height.clamp(0., panel_size_limit.y.max(0.));

        let (top, right, bottom, left) = match anchor {
            DropdownPanelAnchor::TopLeft => (
                Val::Auto,
                Val::Auto,
                Val::Px(dropdown_size.y - dropdown_borders.z),
                Val::Px(-dropdown_borders.w),
            ),
            DropdownPanelAnchor::TopRight => (
                Val::Auto,
                Val::Px(-dropdown_borders.y),
                Val::Px(dropdown_size.y - dropdown_borders.z),
                Val::Auto,
            ),
            DropdownPanelAnchor::BottomLeft => (
                Val::Px(dropdown_size.y - dropdown_borders.x),
                Val::Auto,
                Val::Auto,
                Val::Px(-dropdown_borders.w),
            ),
            DropdownPanelAnchor::BottomRight => (
                Val::Px(dropdown_size.y - dropdown_borders.x),
                Val::Px(-dropdown_borders.y),
                Val::Auto,
                Val::Auto,
            ),
        };

        Ok(DropdownPanelPlacement {
            anchor,
            top,
            right,
            bottom,
            left,
            width: Val::Px(panel_width),
            height: Val::Px(idle_height),
            panel_width: panel_width,
            button_width: dropdown_size.x,
            wider_than_button: panel_width > dropdown_size.x,
        })
    }

    fn button(options: Vec<String>) -> impl Bundle {
        (
            Name::new("Dropdown"),
            ButtonBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    overflow: Overflow::visible(),
                    ..default()
                },
                ..default()
            },
            TrackedInteraction::default(),
            LockedStyleAttributes::from_vec(vec![
                LockableStyleAttribute::FlexDirection,
                LockableStyleAttribute::Overflow,
            ]),
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
            LockedStyleAttributes::lock(LockableStyleAttribute::FocusPolicy),
        )
    }

    fn option_bundle(option: usize) -> impl Bundle {
        (
            Name::new(format!("Option {}", option)),
            ButtonBundle {
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            TrackedInteraction::default(),
            LockedStyleAttributes::lock(LockableStyleAttribute::FocusPolicy),
        )
    }
}

pub trait UiDropdownExt {
    fn dropdown(
        &mut self,
        options: Vec<impl Into<String>>,
        value: impl Into<Option<usize>>,
    ) -> UiBuilder<Entity>;
}

impl UiDropdownExt for UiBuilder<'_, Entity> {
    /// A simple dropdown with options.
    ///
    /// ### PseudoState usage
    /// - `PseudoState::Open`, when the options panel should be visible
    fn dropdown(
        &mut self,
        options: Vec<impl Into<String>>,
        value: impl Into<Option<usize>>,
    ) -> UiBuilder<Entity> {
        let mut label_id = Entity::PLACEHOLDER;
        let mut icon_id = Entity::PLACEHOLDER;
        let mut panel_id = Entity::PLACEHOLDER;
        let mut scroll_view_id = Entity::PLACEHOLDER;
        let mut scroll_view_content_id = Entity::PLACEHOLDER;

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
                    scroll_view_id = container
                        .scroll_view(None, |scroll_view| {
                            scroll_view_content_id = scroll_view.id();

                            for (index, label) in string_options.iter().enumerate() {
                                let mut label_id = Entity::PLACEHOLDER;
                                scroll_view.container(Dropdown::option_bundle(index), |option| {
                                    label_id = option
                                        .label(LabelConfig {
                                            label: label.clone(),
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
                        })
                        .insert(TrackedStyleState::default())
                        .id();
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
                    PseudoStates::default(),
                    VisibilityToPseudoState,
                ))
                .style_unchecked()
                .focus_policy(bevy::ui::FocusPolicy::Block)
                .id();
        });

        dropdown.insert(Dropdown {
            value: value.into(),
            label: label_id,
            icon: icon_id,
            panel: panel_id,
            scroll_view: scroll_view_id,
            scroll_view_content: scroll_view_content_id,
            ..default()
        });

        dropdown
    }
}
