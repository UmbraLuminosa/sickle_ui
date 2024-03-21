use bevy::{ecs::system::CommandQueue, prelude::*, window::PrimaryWindow};

use crate::{
    ui_builder::{UiBuilder, UiBuilderExt, UiContextRoot},
    ui_style::{SetAbsolutePositionExt, UiStyleExt},
    FluxInteractionUpdate,
};

use super::prelude::UiMenuItemSeparatorExt;

const MENU_CONTAINER_Z_INDEX: i32 = 100002;

// TODO: Implement scroll container and up/down arrows when content larger than screen
pub struct ContextMenuPlugin;

impl Plugin for ContextMenuPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, ContextMenuUpdate.after(FluxInteractionUpdate))
            .add_systems(
                Update,
                (
                    update_context_menu_vertical_position,
                    handle_click_or_touch,
                    delete_closed_context_menu,
                    generate_context_menu,
                    position_added_context_menu,
                )
                    .chain()
                    .in_set(ContextMenuUpdate),
            )
            .add_systems(PostUpdate, delete_orphaned_context_menus);
    }
}

// TODO: Handle long press on touch
fn handle_click_or_touch(
    r_mouse: Res<ButtonInput<MouseButton>>,
    q_context_menu: Query<&Interaction, (With<ContextMenu>, Changed<Interaction>)>,
    mut q_interacted: Query<(Entity, &Interaction, &mut GenerateContextMenu)>,
    mut commands: Commands,
) {
    let mut close_all = false;

    if r_mouse.just_pressed(MouseButton::Right) {
        let mut open: Option<Entity> = None;
        for (entity, interaction, _) in &q_interacted {
            if *interaction == Interaction::Hovered {
                open = entity.into();
                break;
            }
        }

        if let Some(open) = open {
            for (entity, _, mut gen_menu) in &mut q_interacted {
                if entity == open {
                    if !gen_menu.is_open {
                        gen_menu.is_open = true;
                    } else if let Some(container) = gen_menu.container {
                        commands.entity(container).despawn_recursive();
                        gen_menu.container = None;
                    }
                } else if gen_menu.is_open {
                    gen_menu.is_open = false;
                }
            }
        } else {
            close_all = true;
        }
    } else if r_mouse.any_just_pressed([MouseButton::Left, MouseButton::Middle]) {
        let mut on_context_menu = false;
        for interaction in &q_context_menu {
            if *interaction == Interaction::Pressed {
                on_context_menu = true;
                break;
            }
        }
        if !on_context_menu {
            close_all = true;
        }
    } else if r_mouse.any_just_released([MouseButton::Left, MouseButton::Middle]) {
        close_all = true;
    }

    if close_all {
        for (_, _, mut gen_menu) in &mut q_interacted {
            if gen_menu.is_open {
                gen_menu.is_open = false;
            }
        }
    }
}

fn delete_closed_context_menu(
    mut q_gen_menus: Query<&mut GenerateContextMenu, Changed<GenerateContextMenu>>,
    mut commands: Commands,
) {
    for mut gen_menu in &mut q_gen_menus {
        if !gen_menu.is_open {
            let Some(container) = gen_menu.container else {
                continue;
            };

            commands.entity(container).despawn_recursive();
            gen_menu.container = None;
        }
    }
}

fn generate_context_menu(world: &mut World) {
    let mut q_gen_menus =
        world.query_filtered::<(Entity, &mut GenerateContextMenu), Changed<GenerateContextMenu>>();

    let mut opened_menu_gen: Option<Entity> = None;
    for (entity, gen_menu) in q_gen_menus.iter(world) {
        if gen_menu.is_open && gen_menu.container.is_none() {
            opened_menu_gen = Some(entity);
            break;
        }
    }

    let Some(entity) = opened_menu_gen else {
        return;
    };

    let mut root_node = entity;
    while let Some(parent) = world.get::<Parent>(root_node) {
        root_node = parent.get();
        if let Some(_) = world.get::<UiContextRoot>(root_node) {
            break;
        }
    }

    let entity_ref = world.entity(entity);
    let type_registry = world.resource::<AppTypeRegistry>().read();
    let mut generators: Vec<&dyn ContextMenuGenerator> = world
        .entity(entity)
        .archetype()
        .components()
        .filter(|component_id| {
            let Some(component_info) = world.components().get_info(*component_id) else {
                return false;
            };

            let Some(type_id) = component_info.type_id() else {
                return false;
            };

            type_registry
                .get_type_data::<ReflectContextMenuGenerator>(type_id)
                .is_some()
        })
        .map(|generator_id| {
            let type_id = world
                .components()
                .get_info(generator_id)
                .unwrap()
                .type_id()
                .unwrap();
            let reflect_generator = type_registry
                .get_type_data::<ReflectContextMenuGenerator>(type_id)
                .unwrap();

            let component = type_registry
                .get(type_id)
                .unwrap()
                .data::<ReflectComponent>()
                .unwrap()
                .reflect(entity_ref)
                .unwrap();

            let actual_generator: &dyn ContextMenuGenerator =
                reflect_generator.get(&*component).unwrap();
            actual_generator
        })
        .collect();

    drop(type_registry);

    if generators.len() == 0 {
        for (orig_entity, mut gen_menu) in q_gen_menus.iter_mut(world) {
            if orig_entity == entity {
                gen_menu.is_open = false;
                break;
            }
        }

        warn!(
            "Cannot create context menu for entity {:?}. No generators implemented!",
            entity
        );
        return;
    }

    generators.sort_by_key(|g| g.placement_index());

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, world);

    let container_id = commands
        .ui_builder(root_node)
        .spawn((ContextMenu::frame(), ContextMenu { context: entity }))
        .id();

    let mut last_index = 0;
    for generator in generators {
        if generator.placement_index() > last_index + 1 {
            commands.ui_builder(container_id).menu_item_separator();
        }
        last_index = generator.placement_index();

        generator.build_context_menu(entity, &mut commands.ui_builder(container_id));
    }

    queue.apply(world);

    for (orig_entity, mut gen_menu) in q_gen_menus.iter_mut(world) {
        if orig_entity == entity {
            gen_menu.container = Some(container_id);
            break;
        }
    }
}

// TODO: Handle touch position
fn position_added_context_menu(
    q_context_menus: Query<Entity, Added<ContextMenu>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    //r_touches: Res<Touches>,
    mut commands: Commands,
) {
    let Ok(window) = q_window.get_single() else {
        return;
    };

    let position = window.cursor_position();
    // if position.is_none() {
    //     position = r_touches.first_pressed_position();
    // }

    let Some(position) = position else {
        return;
    };

    for entity in &q_context_menus {
        commands.style(entity).absolute_position(position);
    }
}

fn update_context_menu_vertical_position(
    mut q_node_style: Query<
        (&Node, &Transform, &mut Style, &mut Visibility),
        (With<ContextMenu>, Changed<Node>),
    >,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = q_window.get_single() else {
        return;
    };

    let resolution = Vec2::new(window.resolution.width(), window.resolution.height());
    for (node, transform, mut style, mut visibility) in &mut q_node_style {
        let size = node.size();

        let position = transform.translation.truncate() - (size / 2.);

        if position.x + size.x > resolution.x {
            style.left = Val::Px(0f32.max(position.x - size.x));
        }
        if position.y + size.y > resolution.y {
            style.top = Val::Px(0f32.max(position.y - size.y));
        }

        *visibility = Visibility::Visible;
    }
}

fn delete_orphaned_context_menus(
    q_context_menus: Query<(Entity, &ContextMenu)>,
    mut commands: Commands,
) {
    for (entity, context_menu) in &q_context_menus {
        if commands.get_entity(context_menu.context).is_none() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct ContextMenuUpdate;

#[reflect_trait]
pub trait ContextMenuGenerator {
    fn build_context_menu(&self, context: Entity, container: &mut UiBuilder<Entity>);
    fn placement_index(&self) -> usize;
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct GenerateContextMenu {
    is_open: bool,
    container: Option<Entity>,
}

impl GenerateContextMenu {
    pub fn is_open(&self) -> bool {
        self.is_open
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ContextMenu {
    context: Entity,
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self {
            context: Entity::PLACEHOLDER,
        }
    }
}

impl ContextMenu {
    pub fn context(&self) -> Entity {
        self.context
    }

    fn frame() -> impl Bundle {
        (
            NodeBundle {
                style: Style {
                    max_height: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    border: UiRect::px(1., 1., 1., 1.),
                    padding: UiRect::px(5., 5., 5., 10.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                z_index: ZIndex::Global(MENU_CONTAINER_Z_INDEX),
                background_color: Color::rgb(0.1, 0.1, 0.1).into(),
                border_color: Color::ANTIQUE_WHITE.into(),
                focus_policy: bevy::ui::FocusPolicy::Block,
                visibility: Visibility::Hidden,
                ..default()
            },
            Interaction::default(),
        )
    }
}
