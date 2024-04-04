use bevy::{ecs::system::CommandQueue, prelude::*};

use crate::{
    ui_builder::{UiBuilder, UiBuilderExt},
    ui_style::{
        SetAlignContentExt, SetAlignItemsExt, SetBackgroundColorExt, SetFlexWrapExt, SetMarginExt,
        SetOverflowExt, SetPaddingExt,
    },
    widgets::prelude::{LabelConfig, UiContainerExt, UiLabelExt, UiRowExt},
};

pub struct EntityComponentListPlugin;

impl Plugin for EntityComponentListPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_entity_component_lists);
    }
}

fn update_entity_component_lists(world: &mut World) {
    let changed: Vec<(Entity, Option<Entity>)> = world
        .query::<(Entity, Ref<EntityComponentList>)>()
        .iter(world)
        .filter(|(_, list)| list.is_changed())
        .map(|(e, list_ref)| (e, list_ref.entity))
        .collect();

    for (container, selected_entity) in changed.iter().copied() {
        update_entity_component_list(container, selected_entity, world);
    }
}

fn update_entity_component_list(
    container: Entity,
    selected_entity: Option<Entity>,
    world: &mut World,
) {
    if let Some(selected) = selected_entity {
        if world.get_entity(selected).is_none() {
            let mut queue = CommandQueue::default();
            let mut commands = Commands::new(&mut queue, world);
            commands.entity(container).despawn_descendants();
            queue.apply(world);
            return;
        }

        let debug_infos: Vec<_> = world
            .inspect_entity(selected)
            .into_iter()
            .map(|component_info| {
                let name = component_info.name();
                let mut simple_name = String::from(name.split("::").last().unwrap());

                if name.split("<").count() > 1 {
                    let left = name.split("<").next().unwrap().split("::").last().unwrap();
                    let generic = name
                        .split("<")
                        .skip(1)
                        .next()
                        .unwrap()
                        .split("::")
                        .last()
                        .unwrap();
                    simple_name = String::new() + left + "<" + generic;
                }

                simple_name
            })
            .collect();

        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, world);
        commands.entity(container).despawn_descendants();
        let mut builder = commands.ui_builder(container);
        for info in debug_infos.iter().cloned() {
            builder
                .container(NodeBundle::default(), |container| {
                    container.label(LabelConfig {
                        label: info,
                        ..default()
                    });
                })
                .style()
                .background_color(Color::GRAY)
                .padding(UiRect::all(Val::Px(3.)))
                .margin(UiRect::all(Val::Px(5.)));
        }
        queue.apply(world);
    } else {
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, world);
        commands.entity(container).despawn_descendants();
        queue.apply(world);
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct EntityComponentList {
    pub entity: Option<Entity>,
}

pub trait UiEntityComponentListExt<'w, 's> {
    fn entity_component_list<'a>(
        &'a mut self,
        entity: Option<Entity>,
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiEntityComponentListExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn entity_component_list<'a>(
        &'a mut self,
        entity: Option<Entity>,
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        self.row(|row| {
            row.insert((
                Name::new("Entity Component List"),
                EntityComponentList { entity },
            ))
            .style()
            .overflow(Overflow::clip())
            .flex_wrap(FlexWrap::Wrap)
            .align_items(AlignItems::FlexStart)
            .align_content(AlignContent::FlexStart);
        })
    }
}
