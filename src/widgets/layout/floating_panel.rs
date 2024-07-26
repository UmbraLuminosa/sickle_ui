use bevy::{
    prelude::*,
    ui::{ContentSize, FocusPolicy, RelativeCursorPosition},
    window::{PrimaryWindow, WindowResized},
};

use sickle_ui_scaffold::{prelude::*, ui_commands::RefreshThemeExt};

use crate::widgets::layout::{
    container::UiContainerExt,
    label::{LabelConfig, SetLabelTextExt, UiLabelExt},
    panel::UiPanelExt,
    resize_handles::{ResizeDirection, ResizeHandle, UiResizeHandlesExt},
    scroll_view::UiScrollViewExt,
};

use super::column::UiColumnExt;

const MIN_PANEL_SIZE: Vec2 = Vec2 { x: 150., y: 100. };
const MIN_FLOATING_PANEL_Z_INDEX: usize = 1000;
const PRIORITY_FLOATING_PANEL_Z_INDEX: usize = 10000;
const WINDOW_RESIZE_PADDING: f32 = 20.;

pub struct FloatingPanelPlugin;

impl Plugin for FloatingPanelPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, FloatingPanelUpdate.after(DroppableUpdate))
            .add_plugins(ComponentThemePlugin::<FloatingPanel>::default())
            .add_systems(PreUpdate, update_floating_panel_panel_id)
            .add_systems(
                Update,
                (
                    index_floating_panels.run_if(panel_added),
                    process_panel_close_pressed,
                    process_panel_fold_pressed,
                    update_panel_size_on_resize,
                    update_panel_on_title_drag,
                    handle_window_resize.run_if(window_resized),
                    update_panel_layout,
                )
                    .chain()
                    .in_set(FloatingPanelUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct FloatingPanelUpdate;

// TODO: Extract widget interaction to separate plugins, i.e. "tab_popout"
// TODO: Re-verify system scheduling, be extra careful of theming not being applied in cases
// when context entity is set late, like below
// TODO: Consider using an observer to do this
fn update_floating_panel_panel_id(
    mut q_floating_panels: Query<
        (Entity, &mut FloatingPanel, &UpdateFloatingPanelPanelId),
        Added<UpdateFloatingPanelPanelId>,
    >,
    mut commands: Commands,
) {
    for (entity, mut floating_panel, update_ref) in &mut q_floating_panels {
        commands
            .entity(entity)
            .remove::<UpdateFloatingPanelPanelId>();

        if update_ref.panel_id == floating_panel.content_panel {
            warn!("Tried setting floating panel id to its current panel!");
            continue;
        }

        commands
            .entity(floating_panel.content_panel)
            .despawn_recursive();

        commands
            .entity(update_ref.panel_id)
            .set_parent(floating_panel.content_panel_container);

        commands.style(update_ref.panel_id).show();

        floating_panel.content_panel = update_ref.panel_id;
        commands.entity(entity).refresh_theme::<FloatingPanel>();
    }
}

fn panel_added(q_panels: Query<Entity, Added<FloatingPanel>>) -> bool {
    q_panels.iter().count() > 0
}

fn index_floating_panels(mut q_panels: Query<&mut FloatingPanel>) {
    let max = if let Some(Some(m)) = q_panels.iter().map(|p| p.z_index).max() {
        m
    } else {
        0
    };

    let mut offset = 1;
    for mut panel in &mut q_panels.iter_mut() {
        if panel.z_index.is_none() {
            panel.z_index = (MIN_FLOATING_PANEL_Z_INDEX + max + offset).into();
            offset += 1;
        }
    }
}

fn process_panel_close_pressed(
    q_buttons: Query<(&FloatingPanelCloseButton, &FluxInteraction), Changed<FluxInteraction>>,
    mut commands: Commands,
) {
    for (button, interaction) in &q_buttons {
        if *interaction == FluxInteraction::Released {
            commands.entity(button.panel).despawn_recursive();
        }
    }
}

fn process_panel_fold_pressed(
    q_buttons: Query<
        (Entity, &FloatingPanelFoldButton, &FluxInteraction),
        Changed<FluxInteraction>,
    >,
    mut q_panel_configs: Query<&mut FloatingPanelConfig>,
) {
    for (entity, button, interaction) in &q_buttons {
        if *interaction == FluxInteraction::Released {
            let Ok(mut config) = q_panel_configs.get_mut(button.panel) else {
                warn!("Missing floating panel config for fold button {}", entity);
                continue;
            };

            config.folded = !config.folded;
        }
    }
}

fn update_panel_size_on_resize(
    q_draggable: Query<(&Draggable, &ResizeHandle, &FloatingPanelResizeHandle), Changed<Draggable>>,
    mut q_panels: Query<&mut FloatingPanel>,
) {
    if let Some(_) = q_panels.iter().find(|p| p.priority) {
        return;
    }

    for (draggable, handle, handle_ref) in &q_draggable {
        let Ok(mut panel) = q_panels.get_mut(handle_ref.panel) else {
            continue;
        };

        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            panel.resizing = false;
            continue;
        }

        let Some(diff) = draggable.diff else {
            continue;
        };

        let size_diff = handle.direction().to_size_diff(diff);

        let old_size = panel.size;
        panel.resizing = true;
        panel.size += size_diff;
        if draggable.state == DragState::DragEnd {
            if panel.size.x < MIN_PANEL_SIZE.x {
                panel.size.x = MIN_PANEL_SIZE.x;
            }
            if panel.size.y < MIN_PANEL_SIZE.y {
                panel.size.y = MIN_PANEL_SIZE.y;
            }
        }

        let pos_diff = match handle.direction() {
            ResizeDirection::North => Vec2 {
                x: 0.,
                y: clip_position_change(diff.y, MIN_PANEL_SIZE.y, old_size.y, panel.size.y),
            },
            ResizeDirection::NorthEast => Vec2 {
                x: 0.,
                y: clip_position_change(diff.y, MIN_PANEL_SIZE.y, old_size.y, panel.size.y),
            },
            ResizeDirection::East => Vec2::ZERO,
            ResizeDirection::SouthEast => Vec2::ZERO,
            ResizeDirection::South => Vec2::ZERO,
            ResizeDirection::SouthWest => Vec2 {
                x: clip_position_change(diff.x, MIN_PANEL_SIZE.x, old_size.x, panel.size.x),
                y: 0.,
            },
            ResizeDirection::West => Vec2 {
                x: clip_position_change(diff.x, MIN_PANEL_SIZE.x, old_size.x, panel.size.x),
                y: 0.,
            },
            ResizeDirection::NorthWest => Vec2 {
                x: clip_position_change(diff.x, MIN_PANEL_SIZE.x, old_size.x, panel.size.x),
                y: clip_position_change(diff.y, MIN_PANEL_SIZE.y, old_size.y, panel.size.y),
            },
        };

        panel.position += pos_diff;
    }
}

fn clip_position_change(diff: f32, min: f32, old_size: f32, new_size: f32) -> f32 {
    let mut new_diff = diff;
    if old_size <= min && new_size <= min {
        new_diff = 0.;
    } else if old_size > min && new_size <= min {
        new_diff -= min - new_size;
    } else if old_size < min && new_size >= min {
        new_diff += min - old_size;
    }

    new_diff
}

fn update_panel_on_title_drag(
    q_draggable: Query<
        (
            &Draggable,
            AnyOf<(&FloatingPanelTitle, &FloatingPanelDragHandle)>,
        ),
        Changed<Draggable>,
    >,
    mut q_panels: Query<(Entity, &mut FloatingPanel)>,
) {
    if let Some(_) = q_panels.iter().find(|(_, p)| p.priority) {
        return;
    }

    let max_index = if let Some(Some(m)) = q_panels.iter().map(|(_, p)| p.z_index).max() {
        m
    } else {
        0
    };
    let mut offset = 1;

    let mut panel_updated = false;

    for (draggable, (panel_title, drag_handle)) in &q_draggable {
        let panel_id = if let Some(panel_title) = panel_title {
            panel_title.panel
        } else if let Some(drag_handle) = drag_handle {
            drag_handle.panel
        } else {
            continue;
        };

        let Ok((_, mut panel)) = q_panels.get_mut(panel_id) else {
            continue;
        };

        if panel.resizing {
            continue;
        }

        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            panel.moving = false;
            continue;
        }

        panel.moving = true;
        let Some(diff) = draggable.diff else {
            continue;
        };

        panel.z_index = Some(max_index + offset);
        panel.position += diff;
        offset += 1;
        panel_updated = true;
    }

    if !panel_updated {
        return;
    }

    let mut panel_indices: Vec<(Entity, Option<usize>)> = q_panels
        .iter()
        .map(|(entity, panel)| (entity, panel.z_index))
        .collect();
    panel_indices.sort_by(|(_, a), (_, b)| a.cmp(b));

    for (i, (entity, _)) in panel_indices.iter().enumerate() {
        if let Some((_, mut panel)) = q_panels.iter_mut().find(|(e, _)| e == entity) {
            panel.z_index = (MIN_FLOATING_PANEL_Z_INDEX + i + 1).into();
        };
    }
}

fn window_resized(e_resized: EventReader<WindowResized>) -> bool {
    e_resized.len() > 0
}

// TODO: Use the panel's render window
fn handle_window_resize(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_panels: Query<(&mut FloatingPanel, &Node, &GlobalTransform)>,
) {
    let Ok(window) = q_window.get_single() else {
        return;
    };

    for (mut panel, node, transform) in &mut q_panels {
        let position = transform.translation().truncate() - (node.size() / 2.);

        if position.x > window.width() - WINDOW_RESIZE_PADDING {
            panel.position.x = (panel.position.x - panel.size.x + WINDOW_RESIZE_PADDING).max(0.);
            if position.y > window.height() - panel.size.y {
                let overflow = position.y - (window.height() - panel.size.y);
                panel.position.y = (panel.position.y - overflow).max(0.);
            }
        }
        if position.y > window.height() - WINDOW_RESIZE_PADDING {
            panel.position.y = (panel.position.y - panel.size.y + WINDOW_RESIZE_PADDING).max(0.);

            if position.x > window.width() - panel.size.x {
                let overflow = position.x - (window.width() - panel.size.x);
                panel.position.x = (panel.position.x - overflow).max(0.);
            }
        }
    }
}

fn update_panel_layout(
    q_panels: Query<
        (Entity, &FloatingPanel, Ref<FloatingPanelConfig>),
        Or<(Changed<FloatingPanel>, Changed<FloatingPanelConfig>)>,
    >,
    mut commands: Commands,
) {
    for (entity, panel, config) in &q_panels {
        if config.is_changed() {
            commands
                .style(panel.title_container)
                .render(config.title.is_some());

            if let Some(title) = config.title.clone() {
                commands.entity(panel.title).set_label_text(title);
                if config.draggable {
                    commands
                        .style(panel.title_container)
                        .enable_flux_interaction();
                } else {
                    commands
                        .style(panel.title_container)
                        .disable_flux_interaction();
                }
            } else {
                commands.style(panel.drag_handle).render(config.draggable);
            }

            commands.style(panel.content_view).render(!config.folded);
            if config.folded {
                commands
                    .entity(entity)
                    .add_pseudo_state(PseudoState::Folded);
            } else {
                commands
                    .entity(entity)
                    .remove_pseudo_state(PseudoState::Folded);
            }
        }

        let render_resize_handles = !config.folded && config.resizable && !panel.moving;
        if render_resize_handles {
            commands
                .entity(panel.resize_handles)
                .insert(PseudoStates::from(vec![
                    PseudoState::Resizable(CardinalDirection::North),
                    PseudoState::Resizable(CardinalDirection::East),
                    PseudoState::Resizable(CardinalDirection::South),
                    PseudoState::Resizable(CardinalDirection::West),
                ]));
        } else {
            commands
                .entity(panel.resize_handles)
                .remove::<PseudoStates>();
        }

        let policy = match panel.moving {
            true => FocusPolicy::Pass,
            false => FocusPolicy::Block,
        };

        commands.style(entity).focus_policy(policy);
        commands
            .style(panel.title_container)
            .focus_policy(policy)
            .flux_interaction_enabled(!panel.resizing);
        commands
            .style(panel.drag_handle)
            .focus_policy(policy)
            .flux_interaction_enabled(!panel.resizing);

        commands
            .style(panel.fold_button)
            .flux_interaction_enabled(!(panel.moving || panel.resizing));
        commands
            .style(panel.close_button)
            .flux_interaction_enabled(!(panel.moving || panel.resizing));

        if panel.resizing {
            commands
                .style(entity)
                .width(Val::Px(panel.size.x.max(MIN_PANEL_SIZE.x)))
                .height(Val::Px(panel.size.y.max(MIN_PANEL_SIZE.y)));
        }

        if panel.moving || panel.resizing {
            commands.style(entity).absolute_position(panel.position);
        }

        if panel.priority {
            commands
                .style(entity)
                .z_index(ZIndex::Global(PRIORITY_FLOATING_PANEL_Z_INDEX as i32));
        } else if let Some(index) = panel.z_index {
            commands.style(entity).z_index(ZIndex::Global(index as i32));
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct FloatingPanelResizeHandle {
    panel: Entity,
}

impl Default for FloatingPanelResizeHandle {
    fn default() -> Self {
        Self {
            panel: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FloatingPanelTitle {
    panel: Entity,
}

impl Default for FloatingPanelTitle {
    fn default() -> Self {
        Self {
            panel: Entity::PLACEHOLDER,
        }
    }
}

impl FloatingPanelTitle {
    pub fn panel(&self) -> Entity {
        self.panel
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FloatingPanelDragHandle {
    panel: Entity,
}

impl Default for FloatingPanelDragHandle {
    fn default() -> Self {
        Self {
            panel: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FloatingPanelFoldButton {
    panel: Entity,
}

impl Default for FloatingPanelFoldButton {
    fn default() -> Self {
        Self {
            panel: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FloatingPanelCloseButton {
    panel: Entity,
}

impl Default for FloatingPanelCloseButton {
    fn default() -> Self {
        Self {
            panel: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct FloatingPanelConfig {
    pub title: Option<String>,
    pub draggable: bool,
    pub resizable: bool,
    pub foldable: bool,
    pub folded: bool,
    pub closable: bool,
    pub restrict_scroll: Option<ScrollAxis>,
}

impl Default for FloatingPanelConfig {
    fn default() -> Self {
        Self {
            title: None,
            draggable: true,
            resizable: true,
            foldable: true,
            folded: false,
            closable: true,
            restrict_scroll: None,
        }
    }
}

impl FloatingPanelConfig {
    pub fn title(&self) -> Option<String> {
        self.title.clone()
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FloatingPanel {
    size: Vec2,
    position: Vec2,
    z_index: Option<usize>,
    drag_handle: Entity,
    fold_button: Entity,
    title_container: Entity,
    title: Entity,
    close_button_container: Entity,
    close_button: Entity,
    content_view: Entity,
    content_panel_container: Entity,
    content_panel: Entity,
    resize_handles: Entity,
    resizing: bool,
    moving: bool,
    pub priority: bool,
}

impl Default for FloatingPanel {
    fn default() -> Self {
        Self {
            size: Default::default(),
            position: Default::default(),
            z_index: Default::default(),
            drag_handle: Entity::PLACEHOLDER,
            fold_button: Entity::PLACEHOLDER,
            title_container: Entity::PLACEHOLDER,
            title: Entity::PLACEHOLDER,
            close_button_container: Entity::PLACEHOLDER,
            close_button: Entity::PLACEHOLDER,
            content_view: Entity::PLACEHOLDER,
            content_panel_container: Entity::PLACEHOLDER,
            content_panel: Entity::PLACEHOLDER,
            resize_handles: Entity::PLACEHOLDER,
            resizing: Default::default(),
            moving: Default::default(),
            priority: Default::default(),
        }
    }
}

impl UiContext for FloatingPanel {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            FloatingPanel::DRAG_HANDLE => Ok(self.drag_handle),
            FloatingPanel::TITLE_CONTAINER => Ok(self.title_container),
            FloatingPanel::TITLE => Ok(self.title),
            FloatingPanel::FOLD_BUTTON => Ok(self.fold_button),
            FloatingPanel::CLOSE_BUTTON_CONTAINER => Ok(self.close_button_container),
            FloatingPanel::CLOSE_BUTTON => Ok(self.close_button),
            FloatingPanel::CONTENT_VIEW => Ok(self.content_view),
            _ => Err(format!(
                "{} doesn't exist for FloatingPanel. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![
            FloatingPanel::DRAG_HANDLE,
            FloatingPanel::TITLE_CONTAINER,
            FloatingPanel::TITLE,
            FloatingPanel::FOLD_BUTTON,
            FloatingPanel::CLOSE_BUTTON_CONTAINER,
            FloatingPanel::CLOSE_BUTTON,
            FloatingPanel::CONTENT_VIEW,
        ]
    }
}

impl DefaultTheme for FloatingPanel {
    fn default_theme() -> Option<Theme<FloatingPanel>> {
        FloatingPanel::theme().into()
    }
}

impl FloatingPanel {
    pub const DRAG_HANDLE: &'static str = "DragHandle";
    pub const TITLE_CONTAINER: &'static str = "TitleContainer";
    pub const TITLE: &'static str = "Title";
    pub const FOLD_BUTTON: &'static str = "FoldButton";
    pub const CLOSE_BUTTON_CONTAINER: &'static str = "CloseButtonContainer";
    pub const CLOSE_BUTTON: &'static str = "CloseButton";
    pub const CONTENT_VIEW: &'static str = "ContentView";

    pub fn theme() -> Theme<FloatingPanel> {
        let base_theme = PseudoTheme::deferred_context(None, FloatingPanel::primary_style);
        let folded_theme =
            PseudoTheme::deferred_context(vec![PseudoState::Folded], FloatingPanel::folded_style);

        Theme::new(vec![base_theme, folded_theme])
    }

    fn primary_style(
        style_builder: &mut StyleBuilder,
        panel: &FloatingPanel,
        theme_data: &ThemeData,
    ) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .absolute_position(panel.position)
            .border(UiRect::all(Val::Px(theme_spacing.borders.extra_small)))
            .border_color(colors.accent(Accent::Shadow))
            .background_color(colors.surface(Surface::Surface))
            .border_radius(BorderRadius::all(Val::Px(
                theme_spacing.corners.extra_small,
            )));

        style_builder
            .animated()
            .height(AnimatedVals {
                idle: Val::Px(panel.size.y.max(MIN_PANEL_SIZE.y)),
                enter_from: Val::Px(theme_spacing.areas.small).into(),
                ..default()
            })
            .copy_from(theme_data.enter_animation);

        style_builder
            .animated()
            .width(AnimatedVals {
                idle: Val::Px(panel.size.x.max(MIN_PANEL_SIZE.x)),
                enter_from: Val::Px(theme_spacing.areas.extra_large).into(),
                ..default()
            })
            .copy_from(theme_data.enter_animation);

        style_builder
            .switch_target(FloatingPanel::TITLE_CONTAINER)
            .width(Val::Percent(100.))
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Start)
            .background_color(colors.container(Container::SurfaceMid))
            .border_radius(BorderRadius::top(Val::Px(
                theme_spacing.corners.extra_small,
            )));

        style_builder
            .switch_target(FloatingPanel::TITLE)
            .flex_grow(1.)
            .margin(UiRect::px(
                theme_spacing.gaps.small,
                theme_spacing.gaps.extra_large,
                theme_spacing.gaps.small,
                theme_spacing.gaps.extra_small,
            ))
            .sized_font(
                theme_data
                    .text
                    .get(FontStyle::Body, FontScale::Large, FontType::Regular),
            )
            .font_color(colors.on(On::Surface));

        style_builder
            .switch_target(FloatingPanel::CLOSE_BUTTON_CONTAINER)
            .right(Val::Px(0.))
            .background_color(colors.container(Container::SurfaceMid))
            .border_radius(BorderRadius::top_right(Val::Px(
                theme_spacing.corners.extra_small,
            )));

        style_builder
            .switch_target(FloatingPanel::CONTENT_VIEW)
            .width(Val::Percent(100.))
            .height(Val::Percent(100.))
            .border_radius(BorderRadius::bottom(Val::Px(
                theme_spacing.corners.extra_small,
            )));

        style_builder
            .switch_context(FloatingPanel::DRAG_HANDLE, None)
            .width(Val::Percent(100.))
            .height(Val::Px(theme_spacing.borders.small * 2.))
            .border(UiRect::bottom(Val::Px(theme_spacing.borders.small)))
            .border_color(colors.accent(Accent::Shadow))
            .animated()
            .background_color(AnimatedVals {
                idle: colors.surface(Surface::Surface),
                hover: colors.surface(Surface::SurfaceVariant).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_context(FloatingPanel::FOLD_BUTTON, None)
            .size(Val::Px(theme_spacing.icons.small))
            .margin(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .icon(
                theme_data
                    .icons
                    .expand_more
                    .with(colors.on(On::Surface), theme_spacing.icons.small),
            )
            .animated()
            .font_color(AnimatedVals {
                idle: colors.on(On::SurfaceVariant),
                hover: colors.on(On::Surface).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_context(FloatingPanel::CLOSE_BUTTON, None)
            .size(Val::Px(theme_spacing.icons.small))
            .margin(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .icon(
                theme_data
                    .icons
                    .close
                    .with(colors.on(On::Surface), theme_spacing.icons.small),
            )
            .animated()
            .font_color(AnimatedVals {
                idle: colors.on(On::SurfaceVariant),
                hover: colors.on(On::Surface).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);
    }

    fn folded_style(
        style_builder: &mut StyleBuilder,
        panel: &FloatingPanel,
        theme_data: &ThemeData,
    ) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .height(Val::Auto)
            .animated()
            .width(AnimatedVals {
                idle: Val::Px(theme_spacing.areas.extra_large),
                enter_from: Val::Px(panel.size.x.max(MIN_PANEL_SIZE.x)).into(),
                ..default()
            })
            .copy_from(theme_data.enter_animation);

        style_builder
            .switch_target(FloatingPanel::CONTENT_VIEW)
            .animated()
            .height(AnimatedVals {
                idle: Val::Percent(0.),
                enter_from: Val::Percent(100.).into(),
                ..default()
            })
            .copy_from(theme_data.enter_animation);

        style_builder
            .switch_target(FloatingPanel::FOLD_BUTTON)
            .icon(
                theme_data
                    .icons
                    .chevron_right
                    .with(colors.on(On::Surface), theme_spacing.icons.small),
            );
    }

    pub fn content_panel_container(&self) -> Entity {
        self.content_panel_container
    }

    pub fn content_panel_id(&self) -> Entity {
        self.content_panel
    }

    pub fn title_container_id(&self) -> Entity {
        self.title_container
    }

    fn frame(title: String) -> impl Bundle {
        (
            Name::new(format!("Floating Panel [{}]", title)),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    overflow: Overflow::clip(),
                    ..default()
                },
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            },
            LockedStyleAttributes::from_vec(vec![
                LockableStyleAttribute::PositionType,
                LockableStyleAttribute::FlexDirection,
                LockableStyleAttribute::AlignItems,
                LockableStyleAttribute::Overflow,
            ]),
        )
    }

    fn title_container(panel: Entity) -> impl Bundle {
        (
            Name::new("Title Container"),
            ButtonBundle::default(),
            FloatingPanelTitle { panel },
            TrackedInteraction::default(),
            Draggable::default(),
            RelativeCursorPosition::default(),
        )
    }

    fn fold_button(panel: Entity) -> impl Bundle {
        (
            Name::new("Fold Button"),
            ButtonBundle::default(),
            ContentSize::default(),
            TrackedInteraction::default(),
            FloatingPanelFoldButton { panel },
        )
    }

    fn drag_handle() -> impl Bundle {
        (
            Name::new("Drag Handle"),
            ButtonBundle::default(),
            TrackedInteraction::default(),
            Draggable::default(),
            RelativeCursorPosition::default(),
        )
    }

    fn close_button_container() -> impl Bundle {
        (
            Name::new("Close Button Container"),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            },
            LockedStyleAttributes::from_vec(vec![
                LockableStyleAttribute::PositionType,
                LockableStyleAttribute::FocusPolicy,
            ]),
        )
    }

    fn close_button(panel: Entity) -> impl Bundle {
        (
            Name::new("Close Button"),
            ButtonBundle::default(),
            ContentSize::default(),
            TrackedInteraction::default(),
            FloatingPanelCloseButton { panel },
        )
    }
}

#[derive(Debug)]
pub struct FloatingPanelLayout {
    pub size: Vec2,
    pub position: Option<Vec2>,
    pub droppable: bool,
}

impl Default for FloatingPanelLayout {
    fn default() -> Self {
        Self {
            size: Vec2 { x: 300., y: 500. },
            position: Default::default(),
            droppable: false,
        }
    }
}

impl FloatingPanelLayout {
    pub fn min() -> Self {
        Self {
            size: MIN_PANEL_SIZE,
            ..default()
        }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct UpdateFloatingPanelPanelId {
    pub panel_id: Entity,
}

pub trait UiFloatingPanelExt {
    fn floating_panel<'a>(
        &'a mut self,
        config: FloatingPanelConfig,
        layout: FloatingPanelLayout,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity>;
}

impl<T: UiContainerExt> UiFloatingPanelExt for T {
    /// A floating panel that can be optionally dragable, foldable, and closable.
    ///
    /// ### PseudoState usage
    /// - `PseudoState::Folded` is used when the panel is folded
    /// - `PseudoState::Resizable(_)` is transiently used by its resize handles
    fn floating_panel<'a>(
        &'a mut self,
        config: FloatingPanelConfig,
        layout: FloatingPanelLayout,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity> {
        let restrict_to = config.restrict_scroll;
        let title_text = if let Some(text) = config.title.clone() {
            text
        } else {
            "Untitled".into()
        };

        let mut floating_panel = FloatingPanel {
            size: layout.size.max(MIN_PANEL_SIZE),
            position: layout.position.unwrap_or_default(),
            z_index: None,
            ..default()
        };

        let mut frame = self.container(FloatingPanel::frame(title_text.clone()), |container| {
            let panel = container.id();
            floating_panel.resize_handles = container
                .resize_handles(FloatingPanelResizeHandle { panel }, |_| {})
                .id();

            let mut title_builder =
                container.container(FloatingPanel::title_container(panel), |container| {
                    floating_panel.fold_button = container
                        .spawn(FloatingPanel::fold_button(panel))
                        .style()
                        .render(config.foldable)
                        .id();

                    floating_panel.title = container
                        .label(LabelConfig {
                            label: title_text.clone(),
                            ..default()
                        })
                        .id();

                    floating_panel.close_button_container = container
                        .container(
                            FloatingPanel::close_button_container(),
                            |close_button_container| {
                                floating_panel.close_button = close_button_container
                                    .spawn(FloatingPanel::close_button(panel))
                                    .style()
                                    .render(config.closable)
                                    .id();
                            },
                        )
                        .id();
                });
            title_builder.style().render(config.title.is_some());

            if layout.droppable {
                title_builder.insert(Droppable);
            }

            floating_panel.title_container = title_builder.id();

            floating_panel.drag_handle = container
                .spawn((
                    FloatingPanel::drag_handle(),
                    FloatingPanelDragHandle { panel },
                ))
                .style()
                .render(config.title.is_none())
                .id();

            floating_panel.content_view = container
                .column(|column| {
                    column.scroll_view(restrict_to, |scroll_view| {
                        floating_panel.content_panel_container = scroll_view.id();
                        floating_panel.content_panel = scroll_view
                            .panel(
                                config.title.clone().unwrap_or("Untitled".into()),
                                spawn_children,
                            )
                            .id();
                    });
                })
                .style()
                .render(config.folded)
                .id();
        });

        if config.folded {
            frame.insert(PseudoStates::from(vec![PseudoState::Folded]));
        }

        frame.insert((config, floating_panel));
        frame
    }
}
