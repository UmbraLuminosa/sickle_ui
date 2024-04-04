use bevy::prelude::*;

use crate::{
    ui_builder::{UiBuilder, UiBuilderExt},
    ui_style::{
        SetBackgroundColorExt, SetBorderColorExt, SetBorderExt, SetMarginExt, SetPaddingExt,
        SetWidthExt, UiStyleExt,
    },
    widgets::{
        foldable::{Foldable, UiFoldableExt},
        prelude::{
            MenuItem, MenuItemConfig, SizedZoneConfig, UiColumnExt, UiMenuItemExt, UiRowExt,
            UiScrollViewExt, UiSizedZoneExt,
        },
    },
};

use super::entity_component_list::{
    EntityComponentList, EntityComponentListPlugin, UiEntityComponentListExt,
};

pub struct HierarchyTreeViewPlugin;
impl Plugin for HierarchyTreeViewPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EntityComponentListPlugin>() {
            app.add_plugins(EntityComponentListPlugin);
        }

        app.add_systems(
            PreUpdate,
            (
                refresh_hierarchy_on_press,
                initialize_hierarchy_tree_view,
                update_hierarchy_on_foldable_change,
                update_hierarchy_selection,
                update_hierarchy_node_style,
                update_entity_component_list,
            )
                .chain()
                .in_set(HierarchyPreUpdate),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct HierarchyPreUpdate;

fn initialize_hierarchy_tree_view(
    q_hierarchy_nodes: Query<(Entity, &HierarchyNodeContainer), Added<HierarchyNodeContainer>>,
    q_hierarchy: Query<&HierarchyContainer>,
    q_name: Query<&Name>,
    mut commands: Commands,
) {
    for (entity, node_container) in &q_hierarchy_nodes {
        let Ok(hierarchy) = q_hierarchy.get(node_container.hierarchy) else {
            warn!(
                "Hierarchy node container {:?} missing main container {:?}",
                entity, node_container.hierarchy
            );
            continue;
        };

        let mut container = commands.ui_builder(entity);
        spawn_hierarchy_level(
            node_container.hierarchy,
            hierarchy.root,
            &mut container,
            &q_name,
        );
    }
}

fn refresh_hierarchy_on_press(
    q_menu_items: Query<(&MenuItem, &RefreshHierarchyButton), Changed<MenuItem>>,
    q_name: Query<&Name>,
    mut q_hierarchy: Query<&mut HierarchyContainer>,
    mut commands: Commands,
) {
    for (menu_item, refresh_button) in &q_menu_items {
        if menu_item.interacted() {
            let Ok(mut hierarchy) = q_hierarchy.get_mut(refresh_button.hierarchy) else {
                continue;
            };

            hierarchy.selected = None;

            commands
                .entity(refresh_button.container)
                .despawn_descendants();

            let mut builder = commands.ui_builder(refresh_button.container);

            spawn_hierarchy_level(
                refresh_button.hierarchy,
                hierarchy.root,
                &mut builder,
                &q_name,
            );

            break;
        }
    }
}

fn update_hierarchy_selection(
    q_menu_items: Query<(&MenuItem, &HierarchyNode), Changed<MenuItem>>,
    mut q_hierarchy: Query<&mut HierarchyContainer>,
) {
    for (menu_item, hierarchy_node) in &q_menu_items {
        if menu_item.interacted() {
            let Ok(mut hierarchy) = q_hierarchy.get_mut(hierarchy_node.hierarchy) else {
                continue;
            };

            if hierarchy.selected != hierarchy_node.entity.into() {
                hierarchy.selected = hierarchy_node.entity.into();
            }
        }
    }
}

fn update_entity_component_list(
    q_hierarchies: Query<&mut HierarchyContainer, Changed<HierarchyContainer>>,
    mut q_entity_component_list: Query<&mut EntityComponentList>,
) {
    for hierarchy in &q_hierarchies {
        let Ok(mut component_list) = q_entity_component_list.get_mut(hierarchy.component_list)
        else {
            continue;
        };

        if component_list.entity != hierarchy.selected {
            component_list.entity = hierarchy.selected;
        }
    }
}

fn update_hierarchy_on_foldable_change(
    mut q_foldables: Query<(&HierarchyNode, &mut MenuItemConfig, &Foldable), Changed<Foldable>>,
    q_children: Query<&Children>,
    q_name: Query<&Name>,
    mut commands: Commands,
) {
    for (hierarchy_node, mut config, foldable) in &mut q_foldables {
        commands.entity(foldable.container()).despawn_descendants();

        if foldable.open {
            if let Ok(children) = q_children.get(hierarchy_node.entity) {
                let mut builder = commands.ui_builder(foldable.container());
                for child in children.iter() {
                    spawn_hierarchy_level(hierarchy_node.hierarchy, *child, &mut builder, &q_name);
                }
            } else {
                config.leading_icon = None;
            }
        } else if q_children.get(hierarchy_node.entity).is_err() {
            config.leading_icon = None;
        }
    }
}

// TODO: Refresh the hierarchy automatically

fn update_hierarchy_node_style(
    q_hierarchies: Query<(Entity, &HierarchyContainer), Changed<HierarchyContainer>>,
    q_hierarchy_nodes: Query<(Entity, &HierarchyNode)>,
    mut commands: Commands,
) {
    for (entity, hierarchy) in &q_hierarchies {
        for (menu_item, hierarchy_node) in q_hierarchy_nodes
            .iter()
            .filter(|(_, node)| node.hierarchy == entity)
        {
            let color = match hierarchy.selected {
                Some(selected) => match hierarchy_node.entity == selected {
                    true => Color::GRAY,
                    false => Color::NONE,
                },
                None => Color::NONE,
            };
            commands.style(menu_item).background_color(color);
        }
    }
}

fn spawn_hierarchy_level(
    hierarchy: Entity,
    entity: Entity,
    container: &mut UiBuilder<'_, '_, '_, Entity>,
    q_name: &Query<&Name>,
) {
    let name = match q_name.get(entity) {
        Ok(name) => format!("[{:?}] {}", entity, name),
        Err(_) => format!("[{:?}]", entity),
    };

    container
        .foldable(name, false, |foldable| {
            foldable
                .style()
                .margin(UiRect::left(Val::Px(10.)))
                .border(UiRect::left(Val::Px(1.)))
                .border_color(Color::rgba(0.98, 0.92, 0.84, 0.25));
        })
        .insert(HierarchyNode { hierarchy, entity });
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct HierarchyNodeContainer {
    hierarchy: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct RefreshHierarchyButton {
    hierarchy: Entity,
    container: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct HierarchyNode {
    hierarchy: Entity,
    entity: Entity,
}

impl HierarchyNode {
    pub fn target(&self) -> Entity {
        self.entity
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct HierarchyContainer {
    root: Entity,
    selected: Option<Entity>,
    component_list: Entity,
}

pub trait UiHierarchyExt<'w, 's> {
    fn hierarchy_for<'a>(&'a mut self, root_entity: Entity) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiHierarchyExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn hierarchy_for<'a>(&'a mut self, root_entity: Entity) -> UiBuilder<'w, 's, 'a, Entity> {
        self.column(|column| {
            column.style().width(Val::Percent(100.));
            let main_zone = column
                .sized_zone(
                    SizedZoneConfig {
                        size: 70.,
                        min_size: 200.,
                    },
                    |zone| {
                        let hierarchy_id = zone.id();
                        let mut refresh_button = Entity::PLACEHOLDER;
                        zone.scroll_view(None, |scroll_view| {
                            scroll_view
                                .row(|row| {
                                    refresh_button = row
                                        .menu_item(MenuItemConfig {
                                            name: "Refresh".into(),
                                            trailing_icon: Some(
                                                "embedded://sickle_ui/icons/redo_white.png".into(),
                                            ),
                                            ..default()
                                        })
                                        .style()
                                        .margin(UiRect::bottom(Val::Px(5.)))
                                        .width(Val::Percent(100.))
                                        .id();
                                })
                                .style()
                                .border(UiRect::bottom(Val::Px(1.)))
                                .margin(UiRect::bottom(Val::Px(10.)))
                                .border_color(Color::ANTIQUE_WHITE);

                            let node_container = scroll_view
                                .column(|_| {})
                                .insert(HierarchyNodeContainer {
                                    hierarchy: hierarchy_id,
                                })
                                .id();

                            scroll_view.commands().entity(refresh_button).insert(
                                RefreshHierarchyButton {
                                    hierarchy: hierarchy_id,
                                    container: node_container,
                                },
                            );
                        });
                    },
                )
                .id();

            let mut component_list = Entity::PLACEHOLDER;
            column.sized_zone(
                SizedZoneConfig {
                    size: 25.,
                    ..default()
                },
                |zone| {
                    component_list = zone.entity_component_list(None).id();
                },
            );

            column
                .commands()
                .ui_builder(main_zone)
                .insert((
                    Name::new(format!("Hierarchy of [{:?}]", root_entity)),
                    HierarchyContainer {
                        root: root_entity,
                        selected: None,
                        component_list,
                    },
                ))
                .style()
                .padding(UiRect::all(Val::Px(5.)));
        })
    }
}
