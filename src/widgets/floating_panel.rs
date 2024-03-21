use bevy::ui::FocusPolicy;
use bevy::window::PrimaryWindow;
use bevy::{prelude::*, window::WindowResized};
use sickle_math::ease::Ease;

use super::icon::UiIconExt;
use super::prelude::{LabelConfig, UiContainerExt, UiLabelExt, UiPanelExt};
use super::prelude::{SetLabelTextExt, UiScrollViewExt};
use crate::animated_interaction::{AnimatedInteraction, AnimationConfig};
use crate::drop_interaction::{Droppable, DroppableUpdate};
use crate::interactions::InteractiveBackground;
use crate::resize_interaction::ResizeHandle;
use crate::ui_builder::UiBuilderExt;
use crate::ui_style::{
    SetAbsolutePositionExt, SetBackgroundColorExt, SetEntityVisiblityExt, SetFluxInteractionExt,
    SetFocusPolicyExt, SetImageExt, SetNodeFlexGrowExt, SetNodeHeightExt, SetNodeMarginExt,
    SetNodeShowHideExt, SetNodeWidthExt, SetZIndexExt, UiStyleExt,
};
use crate::FluxInteraction;
use crate::{
    drag_interaction::{DragState, Draggable},
    resize_interaction::ResizeDirection,
    scroll_interaction::ScrollAxis,
    ui_builder::UiBuilder,
    TrackedInteraction,
};

const MIN_PANEL_SIZE: Vec2 = Vec2 { x: 150., y: 100. };
const MIN_FLOATING_PANEL_Z_INDEX: usize = 1000;
const PRIORITY_FLOATING_PANEL_Z_INDEX: usize = 10000;
const WINDOW_RESIZE_PADDING: f32 = 20.;

pub struct FloatingPanelPlugin;

impl Plugin for FloatingPanelPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, FloatingPanelUpdate.after(DroppableUpdate))
            .add_systems(PreUpdate, update_floating_panel_panel_id)
            .add_systems(
                Update,
                (
                    index_floating_panel.run_if(panel_added),
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

// TODO: Disable resizing when a panel is dragged or resized
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
    }
}

fn panel_added(q_panels: Query<Entity, Added<FloatingPanel>>) -> bool {
    q_panels.iter().count() > 0
}

fn index_floating_panel(mut q_panels: Query<&mut FloatingPanel>) {
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
                warn!("Missing floating panel config for fold button {:?}", entity);
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
            commands
                .style(panel.fold_button)
                .image(match config.folded {
                    true => "sickle_ui://icons/chevron_right.png",
                    false => "sickle_ui://icons/chevron_down.png",
                });
        }

        let render_resize_handles = !config.folded && config.resizable && !panel.moving;
        commands
            .style(panel.resize_handles.0)
            .render(render_resize_handles);
        commands
            .style(panel.resize_handles.1)
            .render(render_resize_handles);

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

        commands
            .style(entity)
            .width(match config.folded {
                true => Val::Auto,
                false => Val::Px(panel.size.x.max(MIN_PANEL_SIZE.x)),
            })
            .height(match config.folded {
                true => Val::Auto,
                false => Val::Px(panel.size.y.max(MIN_PANEL_SIZE.y)),
            })
            .absolute_position(panel.position);

        if panel.priority {
            commands
                .style(entity)
                .z_index(ZIndex::Global(PRIORITY_FLOATING_PANEL_Z_INDEX as i32));
        } else if let Some(index) = panel.z_index {
            commands.style(entity).z_index(ZIndex::Global(index as i32));
        }
    }
}

#[derive(Component, Debug, Reflect)]
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

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct FloatingPanel {
    size: Vec2,
    position: Vec2,
    z_index: Option<usize>,
    own_id: Entity,
    drag_handle: Entity,
    fold_button: Entity,
    title_container: Entity,
    title: Entity,
    close_button: Entity,
    content_view: Entity,
    content_panel_container: Entity,
    content_panel: Entity,
    resize_handles: (Entity, Entity),
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
            own_id: Entity::PLACEHOLDER,
            drag_handle: Entity::PLACEHOLDER,
            fold_button: Entity::PLACEHOLDER,
            title_container: Entity::PLACEHOLDER,
            title: Entity::PLACEHOLDER,
            close_button: Entity::PLACEHOLDER,
            content_view: Entity::PLACEHOLDER,
            content_panel_container: Entity::PLACEHOLDER,
            content_panel: Entity::PLACEHOLDER,
            resize_handles: (Entity::PLACEHOLDER, Entity::PLACEHOLDER),
            resizing: Default::default(),
            moving: Default::default(),
            priority: Default::default(),
        }
    }
}

impl FloatingPanel {
    pub fn content_panel_container(&self) -> Entity {
        self.content_panel_container
    }

    pub fn content_panel_id(&self) -> Entity {
        self.content_panel
    }

    pub fn title_container_id(&self) -> Entity {
        self.title_container
    }

    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                border: UiRect::all(Val::Px(2.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                overflow: Overflow::clip(),
                ..default()
            },
            border_color: Color::BLACK.into(),
            background_color: Color::rgb(0.15, 0.155, 0.16).into(),
            focus_policy: bevy::ui::FocusPolicy::Block,
            ..default()
        }
    }

    fn title_container() -> impl Bundle {
        ButtonBundle {
            style: Style {
                width: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                ..default()
            },
            background_color: Color::DARK_GRAY.into(),
            ..default()
        }
    }

    fn drag_handle() -> impl Bundle {
        (
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Px(6.),
                    border: UiRect::vertical(Val::Px(2.)),
                    ..default()
                },
                border_color: Color::GRAY.into(),
                background_color: Color::BLACK.into(),
                ..default()
            },
            TrackedInteraction::default(),
            Draggable::default(),
        )
    }

    fn close_button_container() -> impl Bundle {
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(0.),
                border: UiRect::left(Val::Px(2.)),
                ..default()
            },
            border_color: Color::BLACK.into(),
            background_color: Color::GRAY.into(),
            focus_policy: bevy::ui::FocusPolicy::Block,
            ..default()
        }
    }
}

#[derive(Debug)]
pub struct FloatingPanelLayout {
    pub size: Vec2,
    pub position: Option<Vec2>,
    pub hidden: bool,
    pub droppable: bool,
}

impl Default for FloatingPanelLayout {
    fn default() -> Self {
        Self {
            size: Vec2 { x: 300., y: 500. },
            position: Default::default(),
            hidden: Default::default(),
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

pub trait UiFloatingPanelSubExt<'w, 's> {
    fn id(&self) -> Entity;
}

impl<'w, 's> UiFloatingPanelSubExt<'w, 's> for UiBuilder<'w, 's, '_, FloatingPanel> {
    fn id(&self) -> Entity {
        self.context().own_id
    }
}

pub trait UiFloatingPanelExt<'w, 's> {
    fn floating_panel<'a>(
        &'a mut self,
        config: FloatingPanelConfig,
        layout: FloatingPanelLayout,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiFloatingPanelExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn floating_panel<'a>(
        &'a mut self,
        config: FloatingPanelConfig,
        layout: FloatingPanelLayout,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let restrict_to = config.restrict_scroll;

        let mut vertical_resize_handles = Entity::PLACEHOLDER;
        let mut horizontal_resize_handles = Entity::PLACEHOLDER;
        let mut title_container = Entity::PLACEHOLDER;
        let mut title = Entity::PLACEHOLDER;
        let mut fold_button = Entity::PLACEHOLDER;
        let mut close_button = Entity::PLACEHOLDER;
        let mut drag_handle = Entity::PLACEHOLDER;
        let mut content_view = Entity::PLACEHOLDER;
        let mut content_panel_container = Entity::PLACEHOLDER;
        let mut content_panel = Entity::PLACEHOLDER;
        let mut frame = self.container(FloatingPanel::frame(), |container| {
            let panel = container.id();

            let title_text = if let Some(text) = config.title.clone() {
                text
            } else {
                "Untitled".into()
            };

            vertical_resize_handles = container
                .container(
                    (ResizeHandle::resize_handle_container(10),),
                    |resize_container| {
                        resize_container.container(
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    height: Val::Px(ResizeHandle::resize_zone_size()),
                                    ..default()
                                },
                                ..default()
                            },
                            |top_row| {
                                top_row.spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::North),
                                    FloatingPanelResizeHandle { panel },
                                ));
                            },
                        );

                        resize_container.container(
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    height: Val::Px(ResizeHandle::resize_zone_size()),
                                    ..default()
                                },
                                ..default()
                            },
                            |bottom_row| {
                                bottom_row.spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::South),
                                    FloatingPanelResizeHandle { panel },
                                ));
                            },
                        );
                    },
                )
                .style()
                .render(config.resizable)
                .id();

            horizontal_resize_handles = container
                .container(
                    (ResizeHandle::resize_handle_container(11),),
                    |resize_container| {
                        resize_container.container(
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    height: Val::Percent(100.),
                                    justify_content: JustifyContent::SpaceBetween,
                                    ..default()
                                },
                                ..default()
                            },
                            |middle_row| {
                                middle_row.spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::West),
                                    FloatingPanelResizeHandle { panel },
                                ));
                                middle_row.spawn((
                                    ResizeHandle::resize_handle(ResizeDirection::East),
                                    FloatingPanelResizeHandle { panel },
                                ));
                            },
                        );
                    },
                )
                .style()
                .render(config.resizable)
                .id();

            let mut title_builder = container.container(
                (
                    FloatingPanel::title_container(),
                    FloatingPanelTitle { panel },
                    TrackedInteraction::default(),
                    Draggable::default(),
                ),
                |container| {
                    fold_button = container
                        .icon(match config.folded {
                            true => "sickle_ui://icons/chevron_right.png",
                            false => "sickle_ui://icons/chevron_down.png",
                        })
                        .insert((
                            Interaction::default(),
                            TrackedInteraction::default(),
                            InteractiveBackground {
                                highlight: Color::rgba(0., 1., 1., 1.).into(),
                                ..default()
                            },
                            AnimatedInteraction::<InteractiveBackground> {
                                tween: FloatingPanel::base_tween(),
                                ..default()
                            },
                            FloatingPanelFoldButton { panel },
                        ))
                        .style()
                        .margin(UiRect::px(3., 0., 3., 3.))
                        .background_color(Color::GRAY)
                        .render(config.foldable)
                        .id();

                    title = container
                        .label(LabelConfig {
                            label: title_text.clone(),
                            margin: UiRect::px(5., 29., 5., 2.),
                            color: Color::ANTIQUE_WHITE,
                            ..default()
                        })
                        .style()
                        .flex_grow(1.)
                        .id();

                    container.container(
                        FloatingPanel::close_button_container(),
                        |close_button_container| {
                            close_button = close_button_container
                                .icon("sickle_ui://icons/close.png")
                                .insert((
                                    Interaction::default(),
                                    TrackedInteraction::default(),
                                    InteractiveBackground {
                                        highlight: Color::rgba(0., 1., 1., 1.).into(),
                                        ..default()
                                    },
                                    AnimatedInteraction::<InteractiveBackground> {
                                        tween: FloatingPanel::base_tween(),
                                        ..default()
                                    },
                                    FloatingPanelCloseButton { panel },
                                ))
                                .style()
                                .margin(UiRect::px(3., 2., 2., 3.))
                                .background_color(Color::rgb(0.1, 0.1, 0.1))
                                .render(config.closable)
                                .id();
                        },
                    );
                },
            );
            title_builder.style().render(config.title.is_some());

            if layout.droppable {
                title_builder.insert(Droppable);
            }

            title_container = title_builder.id();

            drag_handle = container
                .spawn((
                    FloatingPanel::drag_handle(),
                    FloatingPanelDragHandle { panel },
                ))
                .style()
                .render(config.title.is_none())
                .id();

            content_view = container
                .scroll_view(restrict_to, |scroll_view| {
                    content_panel_container = scroll_view.id();
                    content_panel = scroll_view
                        .panel(
                            config.title.clone().unwrap_or("Untitled".into()),
                            spawn_children,
                        )
                        .id();
                })
                .id();
        });

        if layout.hidden {
            frame.style().visibility(Visibility::Hidden);
        }

        let own_id = frame.id();
        let floating_panel = FloatingPanel {
            size: layout.size.max(MIN_PANEL_SIZE),
            position: layout.position.unwrap_or_default(),
            z_index: None,
            own_id,
            drag_handle,
            fold_button,
            title_container,
            title,
            close_button,
            content_view,
            content_panel_container,
            content_panel,
            resize_handles: (horizontal_resize_handles, vertical_resize_handles),
            priority: false,
            ..default()
        };

        frame.insert((config, floating_panel));

        self.commands().ui_builder(own_id)
    }
}
