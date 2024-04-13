//! An example using the widget library to test docking zones and zone splits.
use bevy::prelude::*;

use sickle_math::ease::Ease;
use sickle_ui::{
    dev_panels::hierarchy::{HierarchyTreeViewPlugin, UiHierarchyExt},
    theme::{DynamicStyleBuilder, PseudoTheme, Theme, ThemeData},
    ui_builder::{UiBuilder, UiBuilderExt, UiContextRoot, UiRoot},
    ui_style::{
        AnimatedBundle, SetBackgroundColorExt, SetBorderExt, SetWidthExt, StaticBundle,
        StyleBuilder,
    },
    widgets::{prelude::*, tab_container::UiTabContainerSubExt},
    SickleUiPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sickle UI -  Docking Zone Splits".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SickleUiPlugin)
        .init_resource::<IconCache>()
        .add_plugins(HierarchyTreeViewPlugin)
        .add_systems(Startup, setup.in_set(UiStartupSet))
        .add_systems(Update, Theme::<ThemeTestBox>::update())
        .run();
}

#[derive(Component)]
pub struct UiCamera;

#[derive(SystemSet, Clone, Hash, Debug, Eq, PartialEq)]
pub struct UiStartupSet;

#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
struct IconCache(Vec<Handle<Image>>);

#[derive(Component)]
pub struct ThemeTestBox;

impl ThemeTestBox {
    fn base_theme() -> Theme<ThemeTestBox> {
        let mut style = StyleBuilder::new();
        style
            .border(UiRect::all(Val::Px(2.)))
            .background_color(Color::BLACK)
            .width(Val::Px(100.))
            .height(Val::Px(100.));

        style
            .interactive()
            .border_color(StaticBundle::new(Color::ALICE_BLUE).hover(Color::BISQUE));

        let base_style = PseudoTheme::new(None, style);

        Theme::<ThemeTestBox>::new(vec![base_style])
    }

    fn override_theme() -> Theme<ThemeTestBox> {
        let base_style = PseudoTheme::new(
            None,
            DynamicStyleBuilder::StyleBuilder(ThemeTestBox::style_builder),
        );

        Theme::<ThemeTestBox>::new(vec![base_style])
    }

    fn style_builder(builder: &mut StyleBuilder, data: &ThemeData) {
        builder
            .animated()
            .border_color(AnimatedBundle::new(Color::ALICE_BLUE).hover(Color::BISQUE))
            .hover(0.3, None, None, None);

        builder
            .animated()
            .background_color(AnimatedBundle {
                base: data.background_color,
                hover: Color::GRAY.into(),
                press: Color::GOLD.into(),
                cancel: Color::RED.into(),
                ..default()
            })
            .hover(0.3, Ease::InOutExpo, None, None)
            .press(0.3, None, None, None)
            .cancel(0.3, None, None, None)
            .cancel_reset(0.3, None, 0.3, None);
    }
}

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
            ThemeTestBox::base_theme(),
        ),
        |container| {
            container
                .style()
                .background_color(Color::rgb(0.7, 0.7, 0.7));

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
                .width(Val::Px(400.))
                .border(UiRect::right(Val::Px(4.)))
                .background_color(Color::rgb(0.15, 0.155, 0.16));

            container.sized_zone(
                SizedZoneConfig {
                    size: 80.,
                    ..default()
                },
                |main_content| {
                    root_entity = main_content
                        .insert(UiContextRoot)
                        .style()
                        .width(Val::Percent(100.))
                        .id();

                    spawn_test_content(main_content);
                },
            );
        },
    );

    commands
        .ui_builder(hierarchy_container)
        .hierarchy_for(root_entity);
}

fn spawn_test_content(container: &mut UiBuilder<'_, '_, '_, Entity>) {
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

                                // TODO: Remove test square once theming is done
                                if i > 0 {
                                    return;
                                }

                                panel.spawn((
                                    NodeBundle::default(),
                                    ThemeTestBox,
                                    ThemeTestBox::override_theme(),
                                ));
                            });
                        }
                    },
                )
                .style()
                .background_color(Color::rgb(0.3, 0.3, 0.3));
            sized_zone.sized_zone(
                SizedZoneConfig {
                    size: 20.,
                    ..default()
                },
                |sized_zone| {
                    sized_zone
                        .style()
                        .background_color(Color::rgb(0.3, 0.3, 0.3));
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
                        .background_color(Color::rgb(0.3, 0.3, 0.3));
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
                        .background_color(Color::rgb(0.5, 0.5, 0.5));
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
                        .background_color(Color::rgb(0.5, 0.5, 0.5));
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
                        .background_color(Color::rgb(0.5, 0.5, 0.5));
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
                        .background_color(Color::rgb(0.5, 0.5, 0.5));
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
                        .background_color(Color::rgb(0.5, 0.5, 0.5));
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
                        .background_color(Color::rgb(0.7, 0.7, 0.7));
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
                        .background_color(Color::rgb(0.7, 0.7, 0.7));
                },
            );
        },
    );
}
