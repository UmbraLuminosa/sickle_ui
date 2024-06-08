use bevy::{
    input::mouse::MouseScrollUnit,
    prelude::*,
    ui::{FocusPolicy, RelativeCursorPosition},
};

use sickle_math::ease::Ease;
use sickle_ui_scaffold::{
    drag_interaction::{DragState, Draggable, DraggableUpdate},
    scroll_interaction::{ScrollAxis, Scrollable, ScrollableUpdate},
    ui_builder::UiBuilder,
    ui_style::{SetPaddingExt, TrackedStyleState, UiStyleExt},
    TrackedInteraction,
};

use crate::{
    animated_interaction::{AnimatedInteraction, AnimationConfig},
    interactions::InteractiveBackground,
};

use super::container::UiContainerExt;

pub struct ScrollViewPlugin;

impl Plugin for ScrollViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_scroll_view_on_tracked_style_state_change,
                update_scroll_view_on_content_change,
                update_scroll_view_on_scroll.after(ScrollableUpdate),
                update_scroll_view_on_drag.after(DraggableUpdate),
                update_scroll_view_offset,
                update_scroll_view_layout.in_set(ScrollViewLayoutUpdate),
            )
                .chain(),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct ScrollViewLayoutUpdate;

fn update_scroll_view_on_tracked_style_state_change(
    mut q_scroll_views: Query<(&mut ScrollView, &TrackedStyleState), Changed<TrackedStyleState>>,
) {
    for (mut scroll_view, state) in &mut q_scroll_views {
        let should_disable =
            *state == TrackedStyleState::Enter || *state == TrackedStyleState::Transitioning;

        if scroll_view.disabled != should_disable {
            scroll_view.disabled = should_disable;
        }
    }
}

fn update_scroll_view_on_content_change(
    q_content: Query<&ScrollViewContent, Changed<Node>>,
    mut q_scroll_view: Query<&mut ScrollView>,
) {
    for content in &q_content {
        let Ok(mut container) = q_scroll_view.get_mut(content.scroll_view) else {
            continue;
        };

        // Touch for change
        container.scroll_offset = container.scroll_offset;
    }
}

fn update_scroll_view_on_scroll(
    q_scrollables: Query<
        (AnyOf<(&ScrollViewViewport, &ScrollBarHandle)>, &Scrollable),
        Changed<Scrollable>,
    >,
    mut q_scroll_view: Query<&mut ScrollView>,
) {
    for ((viewport, handle), scrollable) in &q_scrollables {
        let Some((axis, diff, unit)) = scrollable.last_change() else {
            continue;
        };

        let scroll_container_id = if let Some(viewport) = viewport {
            viewport.scroll_view
        } else if let Some(handle) = handle {
            handle.scroll_view
        } else {
            continue;
        };

        let Ok(mut scroll_view) = q_scroll_view.get_mut(scroll_container_id) else {
            continue;
        };

        if scroll_view.disabled {
            continue;
        }

        let offset = match axis {
            ScrollAxis::Horizontal => Vec2 { x: diff, y: 0. },
            ScrollAxis::Vertical => Vec2 { x: 0., y: diff },
        };
        let diff = match unit {
            MouseScrollUnit::Line => offset * 20.,
            MouseScrollUnit::Pixel => offset,
        };
        scroll_view.scroll_offset = scroll_view.scroll_offset + diff;
    }
}

fn update_scroll_view_on_drag(
    q_draggable: Query<(Entity, &Draggable, &ScrollBarHandle), Changed<Draggable>>,
    q_node: Query<&Node>,
    mut q_scroll_view: Query<&mut ScrollView>,
) {
    for (entity, draggable, bar_handle) in &q_draggable {
        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            continue;
        }

        let Ok(mut scroll_view) = q_scroll_view.get_mut(bar_handle.scroll_view) else {
            continue;
        };
        if scroll_view.disabled {
            continue;
        }

        let Some(diff) = draggable.diff else {
            continue;
        };
        let Ok(bar_node) = q_node.get(entity) else {
            continue;
        };
        let Ok(content_node) = q_node.get(scroll_view.content_container) else {
            continue;
        };
        let Ok(container_node) = q_node.get(bar_handle.scroll_view) else {
            continue;
        };

        let container_size = match bar_handle.axis {
            ScrollAxis::Horizontal => container_node.unrounded_size().x,
            ScrollAxis::Vertical => container_node.unrounded_size().y,
        };
        let content_size = match bar_handle.axis {
            ScrollAxis::Horizontal => content_node.unrounded_size().x,
            ScrollAxis::Vertical => content_node.unrounded_size().y,
        };
        let overflow = content_size - container_size;
        if overflow <= 0. {
            continue;
        }

        let bar_size = match bar_handle.axis {
            ScrollAxis::Horizontal => bar_node.unrounded_size().x,
            ScrollAxis::Vertical => bar_node.unrounded_size().y,
        };
        let remaining_space = container_size - bar_size;
        let ratio = overflow / remaining_space;
        let diff = match bar_handle.axis {
            ScrollAxis::Horizontal => diff.x,
            ScrollAxis::Vertical => diff.y,
        } * ratio;

        scroll_view.scroll_offset += match bar_handle.axis {
            ScrollAxis::Horizontal => Vec2 { x: diff, y: 0. },
            ScrollAxis::Vertical => Vec2 { x: 0., y: diff },
        };
    }
}

fn update_scroll_view_offset(
    mut q_scroll_view: Query<(Entity, &mut ScrollView), Changed<ScrollView>>,
    q_node: Query<&Node>,
) {
    for (entity, mut scroll_view) in &mut q_scroll_view {
        let Ok(container_node) = q_node.get(entity) else {
            continue;
        };

        let container_width = container_node.unrounded_size().x;
        let container_height = container_node.unrounded_size().y;
        if container_width == 0. || container_height == 0. {
            continue;
        }

        let Ok(content_node) = q_node.get(scroll_view.content_container) else {
            continue;
        };

        let content_width = content_node.unrounded_size().x;
        let content_height = content_node.unrounded_size().y;

        let overflow_x = content_width - container_width;
        let scroll_offset_x = if overflow_x > 0. {
            scroll_view.scroll_offset.x.clamp(0., overflow_x)
        } else {
            scroll_view.scroll_offset.x
        };
        let overflow_y = content_height - container_height;
        let scroll_offset_y = if overflow_y > 0. {
            scroll_view.scroll_offset.y.clamp(0., overflow_y)
        } else {
            scroll_view.scroll_offset.y
        };

        scroll_view.scroll_offset = Vec2 {
            x: scroll_offset_x,
            y: scroll_offset_y,
        };
    }
}

fn update_scroll_view_layout(
    q_scroll_view: Query<(Entity, &ScrollView), Or<(Changed<ScrollView>, Changed<Node>)>>,
    mut q_node: Query<&Node>,
    mut q_style: Query<&mut Style>,
    mut q_visibility: Query<&mut Visibility>,
    mut commands: Commands,
) {
    for (entity, scroll_view) in &q_scroll_view {
        let Ok(container_node) = q_node.get(entity) else {
            continue;
        };

        let container_width = container_node.unrounded_size().x;
        let container_height = container_node.unrounded_size().y;
        if container_width == 0. || container_height == 0. {
            continue;
        }

        let Ok(content_node) = q_node.get_mut(scroll_view.content_container) else {
            continue;
        };
        let Ok(mut content_style) = q_style.get_mut(scroll_view.content_container) else {
            continue;
        };

        let content_width = content_node.unrounded_size().x;
        let content_height = content_node.unrounded_size().y;

        let overflow_x = content_width - container_width;
        let overflow_y = content_height - container_height;

        // Update content scroll
        if content_height > container_height {
            let scroll_offset_y = scroll_view.scroll_offset.y.clamp(0., overflow_y);
            content_style.top = Val::Px(-scroll_offset_y);
        } else {
            content_style.top = Val::Px(0.);
        }
        if content_width > container_width {
            let scroll_offset_x = scroll_view.scroll_offset.x.clamp(0., overflow_x);
            content_style.left = Val::Px(-scroll_offset_x);
        } else {
            content_style.left = Val::Px(0.);
        }

        let mut padding = (true, true);
        // Update vertical scroll bar
        if let (Some(vertical_scroll_bar), Some(vertical_scroll_bar_handle)) = (
            scroll_view.vertical_scroll_bar,
            scroll_view.vertical_scroll_bar_handle,
        ) {
            if let Ok(mut vertical_bar_visibility) = q_visibility.get_mut(vertical_scroll_bar) {
                if scroll_view.disabled
                    || container_height >= content_height
                    || container_height <= 5.
                {
                    *vertical_bar_visibility = Visibility::Hidden;
                    padding.0 = false;
                } else {
                    *vertical_bar_visibility = Visibility::Inherited;

                    if let Ok(mut handle_style) = q_style.get_mut(vertical_scroll_bar_handle) {
                        let scroll_offset_y = scroll_view.scroll_offset.y.clamp(0., overflow_y);
                        let visible_ratio = (container_height / content_height).clamp(0., 1.);
                        let bar_height =
                            (visible_ratio * container_height).clamp(5., container_height);
                        let remaining_space = container_height - bar_height;
                        let bar_offset = (scroll_offset_y / overflow_y) * remaining_space;

                        handle_style.height = Val::Px(bar_height);
                        handle_style.top = Val::Px(bar_offset);
                    };
                }
            };
        }

        if let (Some(horizontal_scroll_bar), Some(horizontal_scroll_bar_handle)) = (
            scroll_view.horizontal_scroll_bar,
            scroll_view.horizontal_scroll_bar_handle,
        ) {
            // Update horizontal scroll bar
            if let Ok(mut horizontal_bar_visibility) = q_visibility.get_mut(horizontal_scroll_bar) {
                if scroll_view.disabled || container_width >= content_width || container_width <= 5.
                {
                    *horizontal_bar_visibility = Visibility::Hidden;
                    padding.1 = false;
                } else {
                    *horizontal_bar_visibility = Visibility::Inherited;

                    if let Ok(mut handle_style) = q_style.get_mut(horizontal_scroll_bar_handle) {
                        let scroll_offset_x = scroll_view.scroll_offset.x.clamp(0., overflow_x);
                        let visible_ratio = (container_width / content_width).clamp(0., 1.);
                        let bar_width =
                            (visible_ratio * container_width).clamp(5., container_width);
                        let remaining_space = container_width - bar_width;
                        let bar_offset = (scroll_offset_x / overflow_x) * remaining_space;

                        handle_style.width = Val::Px(bar_width);
                        handle_style.left = Val::Px(bar_offset);
                    };
                }
            };
        }

        commands
            .style(scroll_view.content_container)
            .padding(UiRect::px(
                0.,
                match padding.0 {
                    true => 12.,
                    false => 0.,
                },
                0.,
                match padding.1 {
                    true => 12.,
                    false => 0.,
                },
            ));
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollBarHandle {
    axis: ScrollAxis,
    scroll_view: Entity,
}

impl Default for ScrollBarHandle {
    fn default() -> Self {
        Self {
            axis: Default::default(),
            scroll_view: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollBar {
    axis: ScrollAxis,
    scroll_view: Entity,
    handle: Entity,
}

impl Default for ScrollBar {
    fn default() -> Self {
        Self {
            axis: Default::default(),
            scroll_view: Entity::PLACEHOLDER,
            handle: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollViewContent {
    scroll_view: Entity,
}

impl Default for ScrollViewContent {
    fn default() -> Self {
        Self {
            scroll_view: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollViewViewport {
    scroll_view: Entity,
}

impl Default for ScrollViewViewport {
    fn default() -> Self {
        Self {
            scroll_view: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollView {
    viewport: Entity,
    content_container: Entity,
    horizontal_scroll_bar: Option<Entity>,
    horizontal_scroll_bar_handle: Option<Entity>,
    vertical_scroll_bar: Option<Entity>,
    vertical_scroll_bar_handle: Option<Entity>,
    scroll_offset: Vec2,
    pub disabled: bool,
}

impl Default for ScrollView {
    fn default() -> Self {
        Self {
            viewport: Entity::PLACEHOLDER,
            content_container: Entity::PLACEHOLDER,
            horizontal_scroll_bar: None,
            horizontal_scroll_bar_handle: None,
            vertical_scroll_bar: None,
            vertical_scroll_bar_handle: None,
            scroll_offset: Vec2::ZERO,
            disabled: false,
        }
    }
}

impl ScrollView {
    pub fn viewport_id(&self) -> Entity {
        self.viewport
    }

    fn base_tween() -> AnimationConfig {
        AnimationConfig {
            duration: 0.1,
            easing: Ease::OutExpo,
            ..default()
        }
    }

    fn frame() -> impl Bundle {
        (
            Name::new("Scroll View"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
        )
    }

    fn viewport() -> impl Bundle {
        (
            Name::new("Viewport"),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    height: Val::Percent(100.),
                    width: Val::Percent(100.),
                    overflow: Overflow::clip(),
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            Interaction::default(),
            Scrollable::default(),
        )
    }

    fn content(scroll_view: Entity, restrict_to: Option<ScrollAxis>) -> impl Bundle {
        let width = if let Some(axis) = restrict_to {
            match axis {
                ScrollAxis::Horizontal => Val::Auto,
                ScrollAxis::Vertical => Val::Percent(100.),
            }
        } else {
            Val::Auto
        };

        let height = if let Some(axis) = restrict_to {
            match axis {
                ScrollAxis::Horizontal => Val::Percent(100.),
                ScrollAxis::Vertical => Val::Auto,
            }
        } else {
            Val::Auto
        };

        let padding = if let Some(axis) = restrict_to {
            match axis {
                ScrollAxis::Horizontal => UiRect::px(0., 12., 0., 0.),
                ScrollAxis::Vertical => UiRect::px(0., 0., 0., 12.),
            }
        } else {
            UiRect::px(0., 12., 0., 12.)
        };

        (
            Name::new("Content"),
            NodeBundle {
                style: Style {
                    width,
                    height,
                    min_width: Val::Percent(100.),
                    min_height: Val::Percent(100.),
                    justify_self: JustifySelf::Start,
                    align_self: AlignSelf::Start,
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 0.,
                    padding,
                    ..default()
                },
                ..default()
            },
            ScrollViewContent { scroll_view },
        )
    }

    fn scroll_bar_container() -> impl Bundle {
        (
            Name::new("Scroll Bar Container"),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::End,
                    align_content: AlignContent::Stretch,
                    ..default()
                },
                z_index: ZIndex::Local(1),
                ..default()
            },
        )
    }

    fn scroll_bar(axis: ScrollAxis) -> impl Bundle {
        (
            Name::new(match axis {
                ScrollAxis::Horizontal => "Horizontal Scroll Bar",
                ScrollAxis::Vertical => "Vertical Scroll Bar",
            }),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: match axis {
                        ScrollAxis::Horizontal => Val::Percent(100.),
                        ScrollAxis::Vertical => Val::Px(12.),
                    },
                    height: match axis {
                        ScrollAxis::Horizontal => Val::Px(12.),
                        ScrollAxis::Vertical => Val::Percent(100.),
                    },
                    flex_direction: match axis {
                        ScrollAxis::Horizontal => FlexDirection::Row,
                        ScrollAxis::Vertical => FlexDirection::Column,
                    },
                    align_self: AlignSelf::End,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                background_color: Color::GRAY.into(),
                visibility: Visibility::Hidden,
                ..default()
            },
        )
    }

    fn scroll_bar_handle(axis: ScrollAxis) -> impl Bundle {
        (
            Name::new("Scroll Bar Handle"),
            ButtonBundle {
                style: Style {
                    width: match axis {
                        ScrollAxis::Horizontal => Val::Auto,
                        ScrollAxis::Vertical => Val::Percent(100.),
                    },
                    height: match axis {
                        ScrollAxis::Horizontal => Val::Percent(100.),
                        ScrollAxis::Vertical => Val::Auto,
                    },
                    ..default()
                },
                background_color: Color::rgba(0., 1., 1., 0.4).into(),
                ..default()
            },
            TrackedInteraction::default(),
            InteractiveBackground {
                highlight: Color::rgba(0., 1., 1., 0.8).into(),
                ..default()
            },
            AnimatedInteraction::<InteractiveBackground> {
                tween: ScrollView::base_tween(),
                ..default()
            },
            Draggable::default(),
            RelativeCursorPosition::default(),
            Scrollable::default(),
        )
    }
}

pub trait UiScrollViewExt<'w, 's> {
    fn scroll_view<'a>(
        &'a mut self,
        restrict_to: impl Into<Option<ScrollAxis>>,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiScrollViewExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn scroll_view<'a>(
        &'a mut self,
        restrict_to: impl Into<Option<ScrollAxis>>,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let restrict_to = restrict_to.into();
        let mut viewport = Entity::PLACEHOLDER;
        let mut content_container = Entity::PLACEHOLDER;
        let mut horizontal_scroll_id: Option<Entity> = None;
        let mut horizontal_scroll_handle_id: Option<Entity> = None;
        let mut vertical_scroll_id: Option<Entity> = None;
        let mut vertical_scroll_handle_id: Option<Entity> = None;

        let mut scroll_view = self.container(ScrollView::frame(), |frame| {
            let scroll_axes = if let Some(restrict_to) = restrict_to {
                vec![restrict_to]
            } else {
                vec![ScrollAxis::Horizontal, ScrollAxis::Vertical]
            };

            let scroll_view_id = frame.id();
            viewport = frame
                .container(
                    (
                        ScrollView::viewport(),
                        ScrollViewViewport {
                            scroll_view: scroll_view_id,
                        },
                    ),
                    |viewport| {
                        content_container = viewport
                            .container(
                                ScrollView::content(scroll_view_id, restrict_to),
                                spawn_children,
                            )
                            .id();
                    },
                )
                .id();

            frame.container(ScrollView::scroll_bar_container(), |scroll_bar_container| {
                for axis in scroll_axes.iter() {
                    let mut handle_id = Entity::PLACEHOLDER;
                    let mut scroll_bar = scroll_bar_container.container(
                        ScrollView::scroll_bar(*axis),
                        |scroll_bar| {
                            handle_id = scroll_bar
                                .spawn((
                                    ScrollView::scroll_bar_handle(*axis),
                                    ScrollBarHandle {
                                        axis: *axis,
                                        scroll_view: scroll_view_id,
                                    },
                                ))
                                .id();
                        },
                    );
                    scroll_bar.insert(ScrollBar {
                        axis: *axis,
                        scroll_view: scroll_view_id,
                        handle: handle_id,
                    });
                    match axis {
                        ScrollAxis::Horizontal => {
                            horizontal_scroll_id = scroll_bar.id().into();
                            horizontal_scroll_handle_id = handle_id.into();
                        }
                        ScrollAxis::Vertical => {
                            vertical_scroll_id = scroll_bar.id().into();
                            vertical_scroll_handle_id = handle_id.into();
                        }
                    }
                }
            });
        });

        scroll_view.insert(ScrollView {
            viewport,
            content_container,
            horizontal_scroll_bar: horizontal_scroll_id,
            horizontal_scroll_bar_handle: horizontal_scroll_handle_id,
            vertical_scroll_bar: vertical_scroll_id,
            vertical_scroll_bar_handle: vertical_scroll_handle_id,
            ..default()
        });

        scroll_view
    }
}
