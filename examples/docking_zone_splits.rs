//! An example using the widget library to test docking zones and zone splits.
use bevy::prelude::*;

use sickle_ui::{
    dev_panels::hierarchy::{HierarchyTreeViewPlugin, UiHierarchyExt},
    ui_builder::{UiBuilder, UiBuilderExt, UiContextRoot, UiRoot},
    ui_style::prelude::*,
    widgets::prelude::*,
    SickleUiPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sickle UI - Docking Zone Splits".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SickleUiPlugin)
        .add_plugins(HierarchyTreeViewPlugin)
        .add_systems(Startup, setup.in_set(UiStartupSet))
        .run();
}

#[derive(Component)]
pub struct UiCamera;

#[derive(SystemSet, Clone, Hash, Debug, Eq, PartialEq)]
pub struct UiStartupSet;

fn setup(mut commands: Commands) {
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
    let mut hierarchy_container = Entity::PLACEHOLDER;
    let mut root_entity = Entity::PLACEHOLDER;
    commands.ui_builder(UiRoot).container(
        (
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
            TargetCamera(main_camera),
        ),
        |container| {
            container
                .style()
                .background_color(Color::srgb(0.7, 0.7, 0.7));

            container
                .sized_zone(
                    SizedZoneConfig {
                        size: 20.,
                        ..default()
                    },
                    |column| {
                        hierarchy_container = column.id();
                    },
                )
                .style()
                .border(UiRect::right(Val::Px(4.)))
                .background_color(Color::srgb(0.15, 0.155, 0.16));

            container.sized_zone(
                SizedZoneConfig {
                    size: 80.,
                    ..default()
                },
                |main_content| {
                    root_entity = main_content.insert(UiContextRoot).style().id();

                    spawn_test_content(main_content);
                },
            );
        },
    );

    commands
        .ui_builder(hierarchy_container)
        .hierarchy_for(root_entity);
}

fn spawn_test_content(container: &mut UiBuilder<'_, Entity>) {
    container.commands().ui_builder(UiRoot).floating_panel(
        FloatingPanelConfig {
            title: Some("Root floating panel".into()),
            ..default()
        },
        FloatingPanelLayout {
            size: Vec2::new(200., 300.),
            position: Vec2::new(100., 100.).into(),
            droppable: true,
        },
        |panel| {
            panel.docking_zone(
                SizedZoneConfig {
                    size: 60.,
                    ..default()
                },
                false,
                |tab_container| {
                    tab_container.add_tab("Floating docking zone".into(), |panel| {
                        panel.label(LabelConfig {
                            label: "Floating docking zone".into(),
                            ..default()
                        });
                    });
                },
            );
        },
    );

    container.sized_zone(
        SizedZoneConfig {
            size: 10.,
            ..default()
        },
        |sized_zone| {
            sized_zone
                .docking_zone(
                    SizedZoneConfig {
                        size: 60.,
                        ..default()
                    },
                    false,
                    |tab_container| {
                        for i in 0..10 {
                            tab_container.add_tab(format!("Tab {}", i).into(), |panel| {
                                panel.label(LabelConfig {
                                    label: format!("Tab {} content", i).into(),
                                    ..default()
                                });
                            });
                        }
                    },
                )
                .style()
                .background_color(Color::srgb(0.3, 0.3, 0.3));
            sized_zone.sized_zone(
                SizedZoneConfig {
                    size: 20.,
                    ..default()
                },
                |sized_zone| {
                    sized_zone
                        .style()
                        .background_color(Color::srgb(0.3, 0.3, 0.3));
                },
            );
            sized_zone.sized_zone(
                SizedZoneConfig {
                    size: 20.,
                    ..default()
                },
                |sized_zone| {
                    sized_zone
                        .style()
                        .background_color(Color::srgb(0.3, 0.3, 0.3));
                },
            );
        },
    );
    container.sized_zone(
        SizedZoneConfig {
            size: 80.,
            ..default()
        },
        |sized_zone| {
            sized_zone.sized_zone(
                SizedZoneConfig {
                    size: 10.,
                    ..default()
                },
                |sized_zone| {
                    sized_zone
                        .style()
                        .background_color(Color::srgb(0.5, 0.5, 0.5));
                },
            );
            sized_zone.docking_zone_split(
                SizedZoneConfig {
                    size: 80.,
                    ..default()
                },
                |zone_split| {
                    zone_split
                        .docking_zone(
                            SizedZoneConfig {
                                size: 20.,
                                ..default()
                            },
                            true,
                            |tab_container| {
                                tab_container.add_tab("Despawnable zone 1".into(), |panel| {
                                    panel.label(LabelConfig {
                                        label: "Despawnable zone 1 content".into(),
                                        ..default()
                                    });
                                });
                            },
                        )
                        .style()
                        .background_color(Color::srgb(0.5, 0.5, 0.5));
                    zone_split
                        .docking_zone(
                            SizedZoneConfig {
                                size: 60.,
                                ..default()
                            },
                            false,
                            |tab_container| {
                                tab_container.add_tab("Static docking zone".into(), |panel| {
                                    panel.label(LabelConfig {
                                        label: "Static docking zone".into(),
                                        ..default()
                                    });
                                });
                            },
                        )
                        .style()
                        .background_color(Color::srgb(0.5, 0.5, 0.5));
                    zone_split
                        .docking_zone(
                            SizedZoneConfig {
                                size: 20.,
                                ..default()
                            },
                            true,
                            |tab_container| {
                                tab_container.add_tab("Despawnable zone 2".into(), |panel| {
                                    panel.label(LabelConfig {
                                        label: "Despawnable zone 2 content".into(),
                                        ..default()
                                    });
                                });
                            },
                        )
                        .style()
                        .background_color(Color::srgb(0.5, 0.5, 0.5));
                },
            );
            sized_zone.sized_zone(
                SizedZoneConfig {
                    size: 10.,
                    ..default()
                },
                |sized_zone| {
                    sized_zone
                        .style()
                        .background_color(Color::srgb(0.5, 0.5, 0.5));
                },
            );
        },
    );
    container.sized_zone(
        SizedZoneConfig {
            size: 10.,
            ..default()
        },
        |sized_zone| {
            sized_zone.sized_zone(
                SizedZoneConfig {
                    size: 50.,
                    ..default()
                },
                |sized_zone| {
                    sized_zone
                        .style()
                        .background_color(Color::srgb(0.7, 0.7, 0.7));
                },
            );
            sized_zone.sized_zone(
                SizedZoneConfig {
                    size: 50.,
                    ..default()
                },
                |sized_zone| {
                    sized_zone
                        .style()
                        .background_color(Color::srgb(0.7, 0.7, 0.7));
                },
            );
        },
    );
}
