use bevy::{
    prelude::*,
    render::camera::{ManualTextureViews, RenderTarget},
    window::{PrimaryWindow, WindowRef, WindowResolution},
};

pub struct UiUtils;

impl UiUtils {
    /// Gets the nearest clipped container, useful for absolutely positioned elements to find a maximum size
    pub fn container_size_and_offset(entity: Entity, world: &mut World) -> (Vec2, Vec2) {
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

        if let Some(render_target) = UiUtils::find_render_target(entity, world) {
            container_size = UiUtils::render_target_size(render_target, world);
        } else {
            container_size =
                UiUtils::resolution_to_vec2(&UiUtils::get_primary_window(world).resolution);
        }

        (container_size, offset)
    }

    pub fn padding_as_px(entity: Entity, world: &mut World) -> Vec4 {
        // Unsafe unwrap: If a UI element doesn't have a Style, we should panic!
        let style = world.get::<Style>(entity).unwrap();
        UiUtils::ui_rect_to_px(style.padding, entity, world)
    }

    pub fn border_as_px(entity: Entity, world: &mut World) -> Vec4 {
        // Unsafe unwrap: If a UI element doesn't have a Style, we should panic!
        let style = world.get::<Style>(entity).unwrap();
        UiUtils::ui_rect_to_px(style.border, entity, world)
    }

    pub fn margin_as_px(entity: Entity, world: &mut World) -> Vec4 {
        // Unsafe unwrap: If a UI element doesn't have a Style, we should panic!
        let style = world.get::<Style>(entity).unwrap();
        UiUtils::ui_rect_to_px(style.margin, entity, world)
    }

    pub fn ui_rect_to_px(rect: UiRect, entity: Entity, world: &mut World) -> Vec4 {
        let viewport_size = if let Some(render_target) = UiUtils::find_render_target(entity, world)
        {
            UiUtils::render_target_size(render_target, world)
        } else {
            UiUtils::resolution_to_vec2(&UiUtils::get_primary_window(world).resolution)
        };

        let parent_size = if let Some(parent) = world.get::<Parent>(entity) {
            let parent_id = parent.get();
            // Unsafe unwrap: If a UI element doesn't have a Node, we should panic!
            world.get::<Node>(parent_id).unwrap().unrounded_size()
        } else {
            viewport_size
        };

        Vec4::new(
            UiUtils::val_to_px(rect.top, parent_size.y, viewport_size),
            UiUtils::val_to_px(rect.right, parent_size.x, viewport_size),
            UiUtils::val_to_px(rect.bottom, parent_size.y, viewport_size),
            UiUtils::val_to_px(rect.left, parent_size.x, viewport_size),
        )
    }

    pub fn val_to_px(value: Val, parent: f32, viewport_size: Vec2) -> f32 {
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

    pub fn find_render_target(entity: Entity, world: &mut World) -> Option<RenderTarget> {
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

    pub fn render_target_size(render_target: RenderTarget, world: &mut World) -> Vec2 {
        match render_target {
            RenderTarget::Window(window) => match window {
                WindowRef::Primary => {
                    UiUtils::resolution_to_vec2(&UiUtils::get_primary_window(world).resolution)
                }
                WindowRef::Entity(window) => {
                    let Some(window) = world.get::<Window>(window) else {
                        return UiUtils::resolution_to_vec2(
                            &UiUtils::get_primary_window(world).resolution,
                        );
                    };

                    UiUtils::resolution_to_vec2(&window.resolution)
                }
            },
            RenderTarget::Image(handle) => {
                let Some(image) = world.resource::<Assets<Image>>().get(handle) else {
                    return UiUtils::resolution_to_vec2(
                        &UiUtils::get_primary_window(world).resolution,
                    );
                };

                image.size_f32()
            }
            RenderTarget::TextureView(tw_handle) => {
                let Some(texture_view) = world.resource::<ManualTextureViews>().get(&tw_handle)
                else {
                    return UiUtils::resolution_to_vec2(
                        &UiUtils::get_primary_window(world).resolution,
                    );
                };

                Vec2::new(texture_view.size.x as f32, texture_view.size.y as f32)
            }
        }
    }

    pub fn get_primary_window(world: &mut World) -> &Window {
        // Unsafe single: don't ask for a primary window if it doesn't exists pls.
        world
            .query_filtered::<&Window, With<PrimaryWindow>>()
            .single(world)
    }

    pub fn resolution_to_vec2(resolution: &WindowResolution) -> Vec2 {
        Vec2::new(resolution.width(), resolution.height())
    }
}
