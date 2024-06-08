//! An example using the widget library to create a simple 3D scene view with a hierarchy browser for the scene asset.
use bevy::prelude::*;

use sickle_ui::{
    dev_panels::{
        hierarchy::{HierarchyTreeViewPlugin, UiHierarchyExt},
        scene_view::{SceneView, SceneViewPlugin, SpawnSceneViewPreUpdate, UiSceneViewExt},
    },
    theme::{
        icons::IconData,
        theme_data::{Contrast, Scheme, ThemeData},
        PseudoTheme, Theme,
    },
    ui_builder::{UiBuilderExt, UiContextRoot, UiRoot},
    ui_commands::SetCursorExt,
    ui_style::{SetBackgroundColorExt, SetHeightExt, SetJustifyContentsExt, SetWidthExt},
    widgets::{menus::extra_menu::UiExtraMenuExt, prelude::*},
    SickleUiPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sickle UI -  Simple Editor".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SickleUiPlugin)
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
            (
                update_current_page,
                handle_theme_data_update,
                handle_theme_switch,
                handle_theme_contrast_select,
            )
                .chain()
                .after(WidgetLibraryUpdate),
        )
        .run();
}

#[derive(Component, Clone)]
pub struct TMP;

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
struct HierarchyPanel;

#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
struct CurrentPage(Page);

#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
struct IconCache(Vec<Handle<Image>>);

#[derive(Component, Debug)]
pub struct ThemeSwitch;

#[derive(Component, Debug)]
pub struct ThemeContrastSelect;

fn setup(
    asset_server: Res<AssetServer>,
    mut icon_cache: ResMut<IconCache>,
    mut commands: Commands,
) {
    // Workaround for disappearing icons when they are despawned and spawned back in during the same frame
    // Should be fixed in Bevy > 0.13
    let icons_to_cache: Vec<&str> = vec![
        "embedded://sickle_ui/icons/checkmark.png",
        "embedded://sickle_ui/icons/chevron_down.png",
        "embedded://sickle_ui/icons/chevron_left.png",
        "embedded://sickle_ui/icons/chevron_right.png",
        "embedded://sickle_ui/icons/chevron_up.png",
        "embedded://sickle_ui/icons/close.png",
        "embedded://sickle_ui/icons/exit_white.png",
        "embedded://sickle_ui/icons/popout_white.png",
        "embedded://sickle_ui/icons/redo_white.png",
        "embedded://sickle_ui/icons/submenu_white.png",
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

        column.menu_bar(|bar| {
            bar.menu(
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

                    menu.separator();

                    let icons = ThemeData::default().icons;
                    menu.menu_item(MenuItemConfig {
                        name: "Exit".into(),
                        leading_icon: icons.exit_to_app,
                        ..default()
                    })
                    .insert(ExitAppButton);
                },
            );
            bar.menu(
                MenuConfig {
                    name: "Use case".into(),
                    alt_code: KeyCode::KeyS.into(),
                    ..default()
                },
                |menu| {
                    menu.menu_item(MenuItemConfig {
                        name: "Standard menu item".into(),
                        ..default()
                    });
                    menu.menu_item(MenuItemConfig {
                        name: "Menu item with leading icon".into(),
                        leading_icon: IconData::Image(
                            "embedded://sickle_ui/icons/details_menu.png".into(),
                            Color::WHITE,
                        ),
                        ..default()
                    });
                    menu.menu_item(MenuItemConfig {
                        name: "Menu item with trailing icon".into(),
                        trailing_icon: IconData::Image(
                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                            Color::WHITE,
                        ),
                        ..default()
                    });

                    menu.menu_item(MenuItemConfig {
                        name: "Menu item with both icons".into(),
                        leading_icon: IconData::Image(
                            "embedded://sickle_ui/icons/details_menu.png".into(),
                            Color::WHITE,
                        ),
                        trailing_icon: IconData::Image(
                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                            Color::WHITE,
                        ),
                        ..default()
                    });

                    menu.separator();

                    menu.toggle_menu_item(ToggleMenuItemConfig {
                        name: "Toggle item".into(),
                        shortcut: vec![KeyCode::ControlLeft, KeyCode::KeyT].into(),
                        ..default()
                    });
                    menu.toggle_menu_item(ToggleMenuItemConfig {
                        name: "Already toggled item".into(),
                        initially_checked: true,
                        ..default()
                    });
                    menu.toggle_menu_item(ToggleMenuItemConfig {
                        name: "Toggle item with trailing icon".into(),
                        trailing_icon: IconData::Image(
                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                            Color::WHITE,
                        ),
                        ..default()
                    });

                    menu.separator();

                    menu.submenu(
                        SubmenuConfig {
                            name: "Submenu".into(),
                            ..default()
                        },
                        |submenu| {
                            submenu.menu_item(MenuItemConfig {
                                name: "Standard menu item".into(),
                                ..default()
                            });
                            submenu.menu_item(MenuItemConfig {
                                name: "Menu item with leading icon".into(),
                                leading_icon: IconData::Image(
                                    "embedded://sickle_ui/icons/details_menu.png".into(),
                                    Color::WHITE,
                                ),
                                ..default()
                            });
                            submenu.menu_item(MenuItemConfig {
                                name: "Menu item with trailing icon".into(),
                                trailing_icon: IconData::Image(
                                    "embedded://sickle_ui/icons/tiles_menu.png".into(),
                                    Color::WHITE,
                                ),
                                ..default()
                            });
                        },
                    );
                },
            );

            bar.menu(
                MenuConfig {
                    name: "Test case".into(),
                    alt_code: KeyCode::KeyS.into(),
                    ..default()
                },
                |menu| {
                    menu.menu_item(MenuItemConfig {
                        name: "Standard menu item".into(),
                        ..default()
                    });
                    menu.menu_item(MenuItemConfig {
                        name: "Menu item with leading icon".into(),
                        leading_icon: IconData::Image(
                            "embedded://sickle_ui/icons/details_menu.png".into(),
                            Color::WHITE,
                        ),
                        ..default()
                    });
                    menu.menu_item(MenuItemConfig {
                        name: "Menu item with trailing icon".into(),
                        trailing_icon: IconData::Image(
                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                            Color::WHITE,
                        ),
                        ..default()
                    });

                    menu.menu_item(MenuItemConfig {
                        name: "Menu item with both icons".into(),
                        leading_icon: IconData::Image(
                            "embedded://sickle_ui/icons/details_menu.png".into(),
                            Color::WHITE,
                        ),
                        trailing_icon: IconData::Image(
                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                            Color::WHITE,
                        ),
                        ..default()
                    });

                    menu.separator();

                    menu.toggle_menu_item(ToggleMenuItemConfig {
                        name: "Toggle item".into(),
                        shortcut: vec![KeyCode::ControlLeft, KeyCode::KeyT].into(),
                        ..default()
                    });
                    menu.toggle_menu_item(ToggleMenuItemConfig {
                        name: "Already toggled item".into(),
                        initially_checked: true,
                        ..default()
                    });
                    menu.toggle_menu_item(ToggleMenuItemConfig {
                        name: "Toggle item with trailing icon".into(),
                        trailing_icon: IconData::Image(
                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                            Color::WHITE,
                        ),
                        ..default()
                    });

                    menu.separator();

                    menu.submenu(
                        SubmenuConfig {
                            name: "Submenu".into(),
                            ..default()
                        },
                        |submenu| {
                            submenu.menu_item(MenuItemConfig {
                                name: "Standard menu item".into(),
                                ..default()
                            });
                            submenu.menu_item(MenuItemConfig {
                                name: "Menu item with leading icon".into(),
                                leading_icon: IconData::Image(
                                    "embedded://sickle_ui/icons/details_menu.png".into(),
                                    Color::WHITE,
                                ),
                                ..default()
                            });
                            submenu.menu_item(MenuItemConfig {
                                name: "Menu item with trailing icon".into(),
                                trailing_icon: IconData::Image(
                                    "embedded://sickle_ui/icons/tiles_menu.png".into(),
                                    Color::WHITE,
                                ),
                                ..default()
                            });

                            submenu.submenu(
                                SubmenuConfig {
                                    name: "Submenu with lead icon".into(),
                                    leading_icon: IconData::Image(
                                        "embedded://sickle_ui/icons/details_menu.png".into(),
                                        Color::WHITE,
                                    ),
                                    ..default()
                                },
                                |submenu| {
                                    submenu.menu_item(MenuItemConfig {
                                        name: "Standard menu item".into(),
                                        ..default()
                                    });
                                    submenu.menu_item(MenuItemConfig {
                                        name: "Menu item with leading icon".into(),
                                        leading_icon: IconData::Image(
                                            "embedded://sickle_ui/icons/details_menu.png".into(),
                                            Color::WHITE,
                                        ),
                                        ..default()
                                    });
                                    submenu.menu_item(MenuItemConfig {
                                        name: "Menu item with trailing icon".into(),
                                        trailing_icon: IconData::Image(
                                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                                            Color::WHITE,
                                        ),
                                        ..default()
                                    });
                                },
                            );
                        },
                    );
                },
            );

            bar.separator();

            bar.extra_menu(|extra| {
                let narrow_dropdown = PseudoTheme::deferred(None, |style_builder, theme_data| {
                    let theme_spacing = theme_data.spacing;
                    style_builder
                        .min_height(Val::Px(theme_spacing.areas.small))
                        .padding(UiRect::axes(
                            Val::Px(theme_spacing.gaps.medium),
                            Val::Px(theme_spacing.gaps.extra_small),
                        ));
                });
                let narrow_theme = Theme::<Dropdown>::new(vec![narrow_dropdown]);

                extra
                    .radio_group(vec!["Light", "Dark"], false)
                    .insert(ThemeSwitch);
                extra
                    .dropdown(vec!["Standard", "Medium Contrast", "High Contrast"], 0)
                    .insert((ThemeContrastSelect, narrow_theme))
                    .style()
                    .width(Val::Px(150.));
            });
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
    mut commands: Commands,
) {
    let root_entity = root_node.single();
    commands.entity(root_entity).despawn_descendants();
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
                            tab_container.add_tab("Placeholder".into(), |placeholder| {
                                placeholder.radio_group(vec!["Light", "Dark"], false);
                                placeholder.row(|row| {
                                    row.style().justify_content(JustifyContent::SpaceBetween);
                                    row.dropdown(
                                        vec![
                                            "Standard",
                                            "Medium Contrast",
                                            "High Contrast - High Contrast",
                                        ],
                                        None,
                                    );

                                    row.dropdown(
                                        vec![
                                            "Standard",
                                            "Medium Contrast",
                                            "High Contrast - High Contrast",
                                        ],
                                        None,
                                    );
                                });

                                placeholder.scroll_view(None, |scroll_view| {
                                    for _ in 0..10 {
                                        scroll_view.row(|row| {
                                            for _ in 0..10 {
                                                row.container(
                                                    NodeBundle {
                                                        style: Style {
                                                            height: Val::Px(50.),
                                                            flex_shrink: 0.,
                                                            border: UiRect::all(Val::Px(1.)),
                                                            ..default()
                                                        },
                                                        background_color: Color::WHITE.into(),
                                                        border_color: Color::BLACK.into(),
                                                        ..default()
                                                    },
                                                    |container| {
                                                        container.label(LabelConfig {
                                                            label: "Test Node".into(),
                                                            color: Color::BLACK,
                                                            ..default()
                                                        });
                                                    },
                                                );
                                            }
                                        });
                                    }
                                });

                                placeholder.row(|row| {
                                    row.style().justify_content(JustifyContent::SpaceBetween);
                                    row.dropdown(
                                        vec![
                                            "Standard",
                                            "Medium Contrast",
                                            "High Contrast - High Contrast",
                                        ],
                                        None,
                                    );

                                    row.dropdown(
                                        vec![
                                            "Standard",
                                            "Medium Contrast",
                                            "High Contrast - High Contrast",
                                        ],
                                        None,
                                    );
                                });
                            });
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

fn handle_theme_data_update(
    theme_data: Res<ThemeData>,
    mut q_theme_switch: Query<&mut RadioGroup, With<ThemeSwitch>>,
    mut q_theme_contrast_select: Query<&mut Dropdown, With<ThemeContrastSelect>>,
) {
    if theme_data.is_changed() {
        let Ok(mut theme_switch) = q_theme_switch.get_single_mut() else {
            return;
        };

        let Ok(mut theme_contrast_select) = q_theme_contrast_select.get_single_mut() else {
            return;
        };

        match theme_data.active_scheme {
            Scheme::Light(contrast) => {
                theme_switch.select(0);
                match contrast {
                    Contrast::Standard => theme_contrast_select.set_value(0),
                    Contrast::Medium => theme_contrast_select.set_value(1),
                    Contrast::High => theme_contrast_select.set_value(2),
                };
            }
            Scheme::Dark(contrast) => {
                theme_switch.select(1);
                match contrast {
                    Contrast::Standard => theme_contrast_select.set_value(0),
                    Contrast::Medium => theme_contrast_select.set_value(1),
                    Contrast::High => theme_contrast_select.set_value(2),
                };
            }
        };
    }
}
fn handle_theme_switch(
    mut theme_data: ResMut<ThemeData>,
    q_theme_switch: Query<&RadioGroup, (With<ThemeSwitch>, Changed<RadioGroup>)>,
    q_theme_contrast_select: Query<&Dropdown, With<ThemeContrastSelect>>,
) {
    let Ok(theme_switch) = q_theme_switch.get_single() else {
        return;
    };

    let Ok(theme_contrast_select) = q_theme_contrast_select.get_single() else {
        return;
    };

    if let Some(scheme) = get_selected_scheme(theme_switch, theme_contrast_select) {
        if theme_data.active_scheme != scheme {
            theme_data.active_scheme = scheme;
        }
    }
}

fn handle_theme_contrast_select(
    mut theme_data: ResMut<ThemeData>,
    q_theme_switch: Query<&RadioGroup, With<ThemeSwitch>>,
    q_theme_contrast_select: Query<&Dropdown, (With<ThemeContrastSelect>, Changed<Dropdown>)>,
) {
    let Ok(theme_contrast_select) = q_theme_contrast_select.get_single() else {
        return;
    };

    let Ok(theme_switch) = q_theme_switch.get_single() else {
        return;
    };

    if let Some(scheme) = get_selected_scheme(theme_switch, theme_contrast_select) {
        if theme_data.active_scheme != scheme {
            theme_data.active_scheme = scheme;
        }
    }
}

fn get_selected_scheme(
    theme_switch: &RadioGroup,
    theme_contrast_select: &Dropdown,
) -> Option<Scheme> {
    let contrast = match theme_contrast_select.value() {
        Some(index) => match index {
            0 => Contrast::Standard,
            1 => Contrast::Medium,
            2 => Contrast::High,
            _ => Contrast::Standard,
        },
        None => Contrast::Standard,
    };

    if let Some(index) = theme_switch.selected() {
        let scheme = match index {
            0 => Scheme::Light(contrast),
            1 => Scheme::Dark(contrast),
            _ => Scheme::Light(contrast),
        };

        Some(scheme)
    } else {
        None
    }
}
