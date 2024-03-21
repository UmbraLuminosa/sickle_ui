//! An example using the widget library to create a simple 3D scene view with a hierarchy browser for the scene asset.
use bevy::prelude::*;

use sickle_ui::{
    dev_panels::{
        hierarchy::{HierarchyTreeViewPlugin, UiHierarchyExt},
        scene_view::{SceneView, SceneViewPlugin, SpawnSceneViewPreUpdate, UiSceneViewExt},
    },
    ui_builder::{UiBuilderExt, UiContextRoot, UiRoot},
    ui_commands::SetCursorExt,
    ui_style::{SetBackgroundColorExt, SetNodeHeightExt, SetNodeWidthExt},
    widgets::{prelude::*, tab_container::UiTabContainerSubExt, WidgetLibraryUpdate},
    SickleUiPlugin,
};

fn main() {
    App::new()
        .add_plugins(SickleUiPlugin)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sickle UI -  Simple Editor".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<CurrentPage>()
        .init_resource::<IconCache>()
        .init_state::<Page>()
        .add_plugins(HierarchyTreeViewPlugin)
        .add_plugins(SceneViewPlugin)
        .add_systems(Startup, setup.in_set(UiStartupSet))
        .add_systems(OnEnter(Page::Layout), layout_showcase)
        .add_systems(OnExit(Page::Layout), clear_content_on_menu_change)
        .add_systems(OnEnter(Page::Playground), interaction_showcase)
        .add_systems(OnExit(Page::Playground), clear_content_on_menu_change)
        .add_systems(PreUpdate, exit_app_on_menu_item)
        .add_systems(
            PreUpdate,
            (spawn_hierarchy_view, despawn_hierarchy_view).after(SpawnSceneViewPreUpdate),
        )
        .add_systems(
            Update,
            (update_current_page,).chain().after(WidgetLibraryUpdate),
        )
        .run();
}

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
pub struct UiMainRootNode;

#[derive(Component)]
pub struct UiFooterRootNode;

#[derive(SystemSet, Clone, Hash, Debug, Eq, PartialEq)]
pub struct UiStartupSet;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect, States, Hash)]
#[reflect(Component)]
enum Page {
    #[default]
    Layout,
    Playground,
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
struct ExitAppButton;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
struct ShowcaseContainer;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
struct ExtraMenu;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
struct HierarchyPanel;

#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
struct CurrentPage(Page);

#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
struct IconCache(Vec<Handle<Image>>);

fn setup(
    asset_server: Res<AssetServer>,
    mut icon_cache: ResMut<IconCache>,
    mut commands: Commands,
) {
    // Workaround for disappearing icons when they are despawned and spawned back in during the same frame
    // Should be fixed in Bevy > 0.13
    let icons_to_cache: Vec<&str> = vec![
        "sickle_ui://icons/checkmark.png",
        "sickle_ui://icons/chevron_down.png",
        "sickle_ui://icons/chevron_left.png",
        "sickle_ui://icons/chevron_right.png",
        "sickle_ui://icons/chevron_up.png",
        "sickle_ui://icons/close.png",
        "sickle_ui://icons/exit_white.png",
        "sickle_ui://icons/popout_white.png",
        "sickle_ui://icons/redo_white.png",
        "sickle_ui://icons/submenu_white.png",
    ];

    for icon in icons_to_cache.iter() {
        icon_cache.0.push(asset_server.load(*icon));
    }

    // The main camera which will render UI
    let main_camera = commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    order: 1,
                    clear_color: Color::BLACK.into(),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0., 30., 0.))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..Default::default()
            },
            UiCamera,
        ))
        .id();

    // Use the UI builder with plain bundles and direct setting of bundle props
    let mut root_entity = Entity::PLACEHOLDER;
    commands.ui_builder(UiRoot).container(
        (
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
            TargetCamera(main_camera),
        ),
        |container| {
            root_entity = container
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        ..default()
                    },
                    UiMainRootNode,
                ))
                .id();

            container.spawn((
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        width: Val::Percent(100.),
                        height: Val::Px(24.),
                        border: UiRect::top(Val::Px(2.)),
                        ..default()
                    },
                    background_color: Color::rgb(0.29, 0.29, 0.29).into(),
                    border_color: Color::rgb(0.25, 0.25, 0.25).into(),
                    ..default()
                },
                UiFooterRootNode,
            ));
        },
    );

    // Use the UI builder of the root entity with styling applied via commands
    commands.ui_builder(root_entity).column(|column| {
        column
            .style()
            .width(Val::Percent(100.))
            .background_color(Color::rgb(0.15, 0.155, 0.16));

        column.row(|row| {
            row.style()
                .height(Val::Px(30.))
                .background_color(Color::rgb(0.1, 0.1, 0.1));

            row.menu(
                MenuConfig {
                    name: "Showcase".into(),
                    alt_code: KeyCode::KeyS.into(),
                    ..default()
                },
                |menu| {
                    menu.menu_item(MenuItemConfig {
                        name: "Layout".into(),
                        shortcut: vec![KeyCode::KeyL].into(),
                        alt_code: KeyCode::KeyL.into(),
                        ..default()
                    })
                    .insert(Page::Layout);
                    menu.menu_item(MenuItemConfig {
                        name: "Interactions".into(),
                        shortcut: vec![KeyCode::ControlLeft, KeyCode::KeyI].into(),
                        alt_code: KeyCode::KeyI.into(),
                        ..default()
                    })
                    .insert(Page::Playground);

                    menu.menu_item_separator();
                    menu.menu_item(MenuItemConfig {
                        name: "Exit".into(),
                        leading_icon: "sickle_ui://icons/exit_white.png".to_string().into(),
                        ..default()
                    })
                    .insert(ExitAppButton);
                },
            );
            row.spawn((
                NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                },
                ExtraMenu,
            ));
        });
        column
            .row(|_| {})
            .insert((ShowcaseContainer, UiContextRoot))
            .style()
            .height(Val::Percent(100.))
            .background_color(Color::NONE);
    });
}

fn exit_app_on_menu_item(
    q_menu_items: Query<&MenuItem, (With<ExitAppButton>, Changed<MenuItem>)>,
    q_windows: Query<Entity, With<Window>>,
    mut commands: Commands,
) {
    let Ok(item) = q_menu_items.get_single() else {
        return;
    };

    if item.interacted() {
        for entity in &q_windows {
            commands.entity(entity).remove::<Window>();
        }
    }
}

fn update_current_page(
    mut next_state: ResMut<NextState<Page>>,
    q_menu_items: Query<(&Page, &MenuItem), Changed<MenuItem>>,
) {
    for (menu_type, menu_item) in &q_menu_items {
        if menu_item.interacted() {
            next_state.set(*menu_type);
        }
    }
}

fn clear_content_on_menu_change(
    root_node: Query<Entity, With<ShowcaseContainer>>,
    q_extra_menu: Query<Entity, With<ExtraMenu>>,
    mut commands: Commands,
) {
    let root_entity = root_node.single();
    let extra_menu = q_extra_menu.single();
    commands.entity(root_entity).despawn_descendants();
    commands.entity(extra_menu).despawn_descendants();
    commands.set_cursor(CursorIcon::Default);
}

fn spawn_hierarchy_view(
    q_added_scene_view: Query<&SceneView, Added<SceneView>>,
    q_hierarchy_panel: Query<Entity, With<HierarchyPanel>>,

    mut commands: Commands,
) {
    for scene_view in &q_added_scene_view {
        let Ok(container) = q_hierarchy_panel.get_single() else {
            return;
        };

        commands.entity(container).despawn_descendants();
        commands
            .ui_builder(container)
            .hierarchy_for(scene_view.asset_root());
        break;
    }
}

fn despawn_hierarchy_view(
    q_hierarchy_panel: Query<Entity, With<HierarchyPanel>>,
    q_removed_scene_view: RemovedComponents<SceneView>,
    mut commands: Commands,
) {
    let Ok(container) = q_hierarchy_panel.get_single() else {
        return;
    };

    if q_removed_scene_view.len() > 0 {
        commands.entity(container).despawn_descendants();
    }
}

fn layout_showcase(root_node: Query<Entity, With<ShowcaseContainer>>, mut commands: Commands) {
    let root_entity = root_node.single();

    commands
        .ui_builder(root_entity)
        .row(|row| {
            row.docking_zone_split(
                SizedZoneConfig {
                    size: 75.,
                    ..default()
                },
                |left_side| {
                    left_side.docking_zone_split(
                        SizedZoneConfig {
                            size: 75.,
                            ..default()
                        },
                        |left_side_top| {
                            left_side_top.docking_zone(
                                SizedZoneConfig {
                                    size: 25.,
                                    ..default()
                                },
                                true,
                                |tab_container| {
                                    tab_container.add_tab("Hierarchy".into(), |panel| {
                                        panel.insert(HierarchyPanel);
                                    });
                                    tab_container.add_tab("Tab 3".into(), |panel| {
                                        panel.label(LabelConfig {
                                            label: "Panel 3".into(),
                                            ..default()
                                        });
                                    });
                                },
                            );
                            left_side_top.docking_zone(
                                SizedZoneConfig {
                                    size: 75.,
                                    ..default()
                                },
                                false,
                                |tab_container| {
                                    tab_container.add_tab("Scene View".into(), |panel| {
                                        panel.scene_view("examples/Low_poly_scene.gltf#Scene0");
                                    });
                                    tab_container.add_tab("Tab 2".into(), |panel| {
                                        panel.label(LabelConfig {
                                            label: "Panel 2".into(),
                                            ..default()
                                        });
                                    });
                                    tab_container.add_tab("Tab 3".into(), |panel| {
                                        panel.label(LabelConfig {
                                            label: "Panel 3".into(),
                                            ..default()
                                        });
                                    });
                                },
                            );
                        },
                    );

                    left_side.docking_zone(
                        SizedZoneConfig {
                            size: 25.,
                            ..default()
                        },
                        true,
                        |tab_container| {
                            tab_container.add_tab("Systems".into(), |panel| {
                                panel.label(LabelConfig {
                                    label: "Systems".into(),
                                    ..default()
                                });
                            });
                            tab_container.add_tab("Tab 6".into(), |panel| {
                                panel.label(LabelConfig {
                                    label: "Panel 6".into(),
                                    ..default()
                                });
                            });
                        },
                    );
                },
            );

            row.docking_zone_split(
                SizedZoneConfig {
                    size: 25.,
                    ..default()
                },
                |right_side| {
                    right_side.docking_zone(
                        SizedZoneConfig {
                            size: 25.,
                            ..default()
                        },
                        true,
                        |tab_container| {
                            tab_container.add_tab("Placeholder".into(), |_| {});
                        },
                    );
                },
            );
        })
        .style()
        .height(Val::Percent(100.));
}

fn interaction_showcase(root_node: Query<Entity, With<ShowcaseContainer>>, mut commands: Commands) {
    let root_entity = root_node.single();

    commands.ui_builder(root_entity).column(|_column| {
        // Test here simply by calling methods on the `column`
    });
}
