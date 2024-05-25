use std::collections::VecDeque;

use bevy::{
    prelude::*,
    render::camera::{ManualTextureViews, RenderTarget},
    ui::FocusPolicy,
    window::{PrimaryWindow, WindowResolution},
};

use crate::{
    theme::{
        pseudo_state::{PseudoState, PseudoStates},
        theme_colors::{Accent, Container, On},
        theme_data::ThemeData,
        typography::{FontScale, FontStyle, FontType},
        ComponentThemePlugin, DefaultTheme, PseudoTheme, Theme, UiContext,
    },
    ui_builder::UiBuilder,
    ui_commands::ManagePseudoStateExt,
    ui_style::{
        AnimatedVals, LockableStyleAttribute, LockedStyleAttributes, SetDisplayUncheckedExt,
        SetFocusPolicyUncheckedExt, SetHeightUncheckedExt, SetVisibilityUncheckedExt, StyleBuilder,
        TrackedStyleState, UiStyleUncheckedExt,
    },
    FluxInteraction, FluxInteractionUpdate, TrackedInteraction,
};

use super::{
    prelude::{LabelConfig, UiContainerExt, UiLabelExt, UiPanelExt, UiScrollViewExt},
    scroll_view::{ScrollView, ScrollViewLayoutUpdate},
};

const DROPDOWN_LABEL: &'static str = "Label";
const DROPDOWN_ICON: &'static str = "Icon";
const DROPDOWN_PANEL: &'static str = "Panel";
const DROPDOWN_SCROLL_VIEW: &'static str = "ScrollView";
const DROPDOWN_OPTION_LABEL: &'static str = "Label";
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
            // TODO: Set text in a way the theming applies the font
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

fn update_drowdown_pseudo_state(
    q_panels: Query<(&DropdownPanel, &PseudoStates), Changed<PseudoStates>>,
    mut commands: Commands,
) {
    for (panel, states) in &q_panels {
        if states.has(PseudoState::Visible) {
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
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct DropdownOptions(Vec<String>);

#[derive(Component, Clone, Debug, Reflect)]
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
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![DROPDOWN_OPTION_LABEL]
    }
}

impl DefaultTheme for DropdownOption {
    fn default_theme() -> Option<Theme<DropdownOption>> {
        DropdownOption::theme().into()
    }
}

impl DropdownOption {
    pub fn theme() -> Theme<DropdownOption> {
        let base_theme = PseudoTheme::deferred(None, DropdownOption::primary_style);

        Theme::<DropdownOption>::new(vec![base_theme])
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
                idle: Color::NONE,
                hover: colors.container(Container::SurfaceHighest).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(DROPDOWN_OPTION_LABEL)
            .sized_font(font)
            .font_color(colors.on(On::Surface));
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
            DROPDOWN_LABEL => Ok(self.label),
            DROPDOWN_ICON => Ok(self.icon),
            DROPDOWN_PANEL => Ok(self.panel),
            DROPDOWN_SCROLL_VIEW => Ok(self.scroll_view),
            _ => Err(format!(
                "{} doesn't exists for Dropdown. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![
            DROPDOWN_LABEL,
            DROPDOWN_ICON,
            DROPDOWN_PANEL,
            DROPDOWN_SCROLL_VIEW,
        ]
    }
}

impl DefaultTheme for Dropdown {
    fn default_theme() -> Option<Theme<Dropdown>> {
        Dropdown::theme().into()
    }
}

impl Dropdown {
    pub fn value(&self) -> Option<usize> {
        self.value
    }

    pub fn theme() -> Theme<Dropdown> {
        let base_theme = PseudoTheme::deferred(None, Dropdown::primary_style);
        let open_theme = PseudoTheme::deferred_world(vec![PseudoState::Open], Dropdown::open_style);

        Theme::<Dropdown>::new(vec![base_theme, open_theme])
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
            .switch_target(DROPDOWN_LABEL)
            .sized_font(font)
            .animated()
            .font_color(AnimatedVals {
                idle: colors.on(On::PrimaryContainer),
                hover: colors.on(On::Surface).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(DROPDOWN_ICON)
            .size(Val::Px(theme_spacing.icons.small))
            .margin(UiRect::left(Val::Px(theme_spacing.gaps.large)))
            .icon(
                theme_data
                    .icons
                    .expand_more
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
            .switch_target(DROPDOWN_PANEL)
            .position_type(PositionType::Absolute)
            .z_index(ZIndex::Global(DROPDOWN_PANEL_Z_INDEX as i32))
            .border(UiRect::all(Val::Px(theme_spacing.gaps.tiny)))
            .border_color(colors.accent(Accent::Shadow))
            .background_color(colors.container(Container::SurfaceMid))
            .top(Val::Px(theme_spacing.areas.medium))
            .min_width(Val::Percent(100.))
            .max_height(Val::Px(theme_spacing.areas.extra_large));
    }

    fn open_style(style_builder: &mut StyleBuilder, entity: Entity, world: &mut World) {
        let placement = match Dropdown::panel_placement_for(entity, world) {
            Ok(placement) => placement,
            Err(msg) => {
                error!("Error placing Dropdown panel: {}", msg);
                return;
            }
        };

        let enter_animation = world.resource::<ThemeData>().enter_animation.clone();

        if placement.anchor == DropdownPanelAnchor::BottomLeft
            || placement.anchor == DropdownPanelAnchor::BottomRight
        {
            style_builder
                .switch_target(DROPDOWN_PANEL)
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
        } else {
            style_builder
                .switch_target(DROPDOWN_PANEL)
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

            style_builder
                .switch_target(DROPDOWN_PANEL)
                .animated()
                .top(AnimatedVals {
                    idle: placement.top,
                    enter_from: Val::Px(0.).into(),
                    ..default()
                })
                .copy_from(enter_animation);
        }

        style_builder
            .switch_target(DROPDOWN_SCROLL_VIEW)
            .animated()
            .tracked_style_state(TrackedStyleState::default_vals())
            .copy_from(enter_animation);
    }

    pub fn panel_placement_for(
        entity: Entity,
        world: &mut World,
    ) -> Result<DropdownPanelPlacement, String> {
        let Some(dropdown) = world.get::<Dropdown>(entity) else {
            return Err("Entity has no Dropdown component".into());
        };
        let dropdown_panel = dropdown.panel;
        let scroll_view_content = dropdown.scroll_view_content;

        // Unsafe unwrap: If a UI element doesn't have a Node, we should panic!
        let dropdown_node = world.get::<Node>(entity).unwrap();
        let dropdown_size = dropdown_node.unrounded_size();
        // TODO: bevy 0.14(?) add calculated border size from component rather than calculate it here
        let dropdown_borders = Dropdown::get_node_border_sizes(entity, world);
        let panel_borders = Dropdown::get_node_border_sizes(dropdown_panel, world);

        let (container_size, dropdown_position) =
            Dropdown::get_container_size_and_offset(entity, world);
        let tl_corner = dropdown_position - dropdown_size / 2.;
        let total_available_space = container_size - dropdown_size;
        let halfway_point = total_available_space / 2.;
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
            DropdownPanelAnchor::TopLeft => {
                Vec2::new(total_available_space.x - tl_corner.x, tl_corner.y)
            }
            DropdownPanelAnchor::TopRight => Vec2::new(tl_corner.x + dropdown_size.x, tl_corner.y),
            DropdownPanelAnchor::BottomLeft => Vec2::new(
                total_available_space.x - tl_corner.x,
                total_available_space.y - (tl_corner.y + dropdown_size.y),
            ),
            DropdownPanelAnchor::BottomRight => Vec2::new(
                tl_corner.x + dropdown_size.x,
                total_available_space.y - (tl_corner.y + dropdown_size.y),
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

                let margin_sizes = Dropdown::get_node_margin_sizes(child, world);
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
                -Val::Px(idle_height + dropdown_borders.x),
                Val::Auto,
                Val::Auto,
                Val::Px(-dropdown_borders.w),
            ),
            DropdownPanelAnchor::TopRight => (
                -Val::Px(idle_height + dropdown_borders.x),
                Val::Px(-dropdown_borders.y),
                Val::Auto,
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
        })
    }

    fn get_node_border_sizes(entity: Entity, world: &mut World) -> Vec4 {
        // Unsafe unwrap: If a UI element doesn't have a Style, we should panic!
        let style = world.get::<Style>(entity).unwrap();
        let border = style.border;

        let viewport_size = if let Some(render_target) = Dropdown::find_render_target(entity, world)
        {
            Dropdown::get_render_target_size(render_target, world)
        } else {
            Dropdown::resolution_to_vec2(&Dropdown::get_primary_window(world).resolution)
        };

        let parent_size = if let Some(parent) = world.get::<Parent>(entity) {
            let parent_id = parent.get();
            // Unsafe unwrap: If a UI element doesn't have a Node, we should panic!
            world.get::<Node>(parent_id).unwrap().unrounded_size()
        } else {
            viewport_size
        };

        Vec4::new(
            Dropdown::val_to_px(border.top, parent_size.y, viewport_size),
            Dropdown::val_to_px(border.right, parent_size.x, viewport_size),
            Dropdown::val_to_px(border.bottom, parent_size.y, viewport_size),
            Dropdown::val_to_px(border.left, parent_size.x, viewport_size),
        )
    }

    // Extract these methods, these should be useful in any case
    fn get_node_margin_sizes(entity: Entity, world: &mut World) -> Vec4 {
        // Unsafe unwrap: If a UI element doesn't have a Style, we should panic!
        let style = world.get::<Style>(entity).unwrap();
        let margin = style.margin;

        let viewport_size = if let Some(render_target) = Dropdown::find_render_target(entity, world)
        {
            Dropdown::get_render_target_size(render_target, world)
        } else {
            Dropdown::resolution_to_vec2(&Dropdown::get_primary_window(world).resolution)
        };

        let parent_size = if let Some(parent) = world.get::<Parent>(entity) {
            let parent_id = parent.get();
            // Unsafe unwrap: If a UI element doesn't have a Node, we should panic!
            world.get::<Node>(parent_id).unwrap().unrounded_size()
        } else {
            viewport_size
        };

        Vec4::new(
            Dropdown::val_to_px(margin.top, parent_size.y, viewport_size),
            Dropdown::val_to_px(margin.right, parent_size.x, viewport_size),
            Dropdown::val_to_px(margin.bottom, parent_size.y, viewport_size),
            Dropdown::val_to_px(margin.left, parent_size.x, viewport_size),
        )
    }

    fn val_to_px(value: Val, parent: f32, viewport_size: Vec2) -> f32 {
        match value {
            Val::Auto => 0.,
            Val::Px(px) => px.max(0.),
            Val::Percent(percent) => (parent * percent / 100.).max(0.),
            Val::Vw(percent) => (viewport_size.x * percent / 100.).max(0.),
            Val::Vh(percent) => (viewport_size.y * percent / 100.).max(0.),
            Val::VMin(percent) => (viewport_size.min_element() * percent / 100.).max(0.),
            Val::VMax(percent) => (viewport_size.max_element() * percent / 100.).max(0.),
        }
    }

    fn get_container_size_and_offset(entity: Entity, world: &mut World) -> (Vec2, Vec2) {
        let mut container_size = Vec2::ZERO;

        // Unsafe unwarp: If a dropdown doesn't have a GT, we should panic!
        let mut offset = world
            .get::<GlobalTransform>(entity)
            .unwrap()
            .translation()
            .truncate();

        let mut current_ancestor = entity;
        while let Some(parent) = world.get::<Parent>(current_ancestor) {
            current_ancestor = parent.get();

            // Unsafe unwrap: If a UI element doesn't have a Style, we should panic!
            let style = world.get::<Style>(current_ancestor).unwrap();
            if style.overflow.x == OverflowAxis::Visible
                && style.overflow.y == OverflowAxis::Visible
            {
                continue;
            }

            // Unsafe unwrap: If a UI element doesn't have a Node, we should panic!
            let node = world.get::<Node>(current_ancestor).unwrap();
            let node_size = node.unrounded_size();
            // Unsafe unwrap: If a UI element doesn't have a GT, we should panic!
            let current_pos = world
                .get::<GlobalTransform>(current_ancestor)
                .unwrap()
                .translation()
                .truncate();

            if container_size.x == 0. && style.overflow.x == OverflowAxis::Clip {
                container_size.x = node_size.x;
                offset.x -= current_pos.x - (node_size.x / 2.);
            }

            if container_size.y == 0. && style.overflow.y == OverflowAxis::Clip {
                container_size.y = node_size.y;
                offset.y -= current_pos.y - (node_size.y / 2.);
            }

            if container_size.x > 0. && container_size.y > 0. {
                return (container_size, offset);
            }
        }

        if let Some(render_target) = Dropdown::find_render_target(entity, world) {
            container_size = Dropdown::get_render_target_size(render_target, world);
        } else {
            container_size =
                Dropdown::resolution_to_vec2(&Dropdown::get_primary_window(world).resolution);
        }

        (container_size, offset)
    }

    fn find_render_target(entity: Entity, world: &mut World) -> Option<RenderTarget> {
        let mut current_ancestor = entity;
        while let Some(parent) = world.get::<Parent>(current_ancestor) {
            current_ancestor = parent.get();
            if let Some(target_camera) = world.get::<TargetCamera>(current_ancestor) {
                let camera_entity = target_camera.0;
                if let Some(camera) = world.get::<Camera>(camera_entity) {
                    return camera.target.clone().into();
                };
            }
        }

        None
    }

    fn get_render_target_size(render_target: RenderTarget, world: &mut World) -> Vec2 {
        match render_target {
            RenderTarget::Window(window) => match window {
                bevy::window::WindowRef::Primary => {
                    Dropdown::resolution_to_vec2(&Dropdown::get_primary_window(world).resolution)
                }
                bevy::window::WindowRef::Entity(window) => {
                    let Some(window) = world.get::<Window>(window) else {
                        return Dropdown::resolution_to_vec2(
                            &Dropdown::get_primary_window(world).resolution,
                        );
                    };

                    Dropdown::resolution_to_vec2(&window.resolution)
                }
            },
            RenderTarget::Image(handle) => {
                let Some(image) = world.resource::<Assets<Image>>().get(handle) else {
                    return Dropdown::resolution_to_vec2(
                        &Dropdown::get_primary_window(world).resolution,
                    );
                };

                image.size_f32()
            }
            RenderTarget::TextureView(tw_handle) => {
                let Some(texture_view) = world.resource::<ManualTextureViews>().get(&tw_handle)
                else {
                    return Dropdown::resolution_to_vec2(
                        &Dropdown::get_primary_window(world).resolution,
                    );
                };

                Vec2::new(texture_view.size.x as f32, texture_view.size.y as f32)
            }
        }
    }

    fn get_primary_window(world: &mut World) -> &Window {
        world
            .query_filtered::<&Window, With<PrimaryWindow>>()
            .single(world)
    }

    fn resolution_to_vec2(resolution: &WindowResolution) -> Vec2 {
        Vec2::new(resolution.width(), resolution.height())
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
            LockedStyleAttributes::lock(LockableStyleAttribute::FlexDirection),
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

pub trait UiDropdownExt<'w, 's> {
    fn dropdown<'a>(
        &'a mut self,
        options: Vec<impl Into<String>>,
        value: impl Into<Option<usize>>,
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiDropdownExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn dropdown<'a>(
        &'a mut self,
        options: Vec<impl Into<String>>,
        value: impl Into<Option<usize>>,
    ) -> UiBuilder<'w, 's, 'a, Entity> {
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
