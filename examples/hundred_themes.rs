//! [Epilepsy WARNING] An example using the widget library to test performance for DynamicStyles and Theme application.
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    time::Stopwatch,
};

use sickle_macros::UiContext;
use sickle_math::ease::Ease;
use sickle_ui::{
    theme::{
        pseudo_state::{PseudoState, PseudoStates},
        style_animation::AnimationLoop,
        theme_data::{Contrast, Scheme, ThemeData},
        ComponentThemePlugin, DefaultTheme, PseudoTheme, Theme, UiContext,
    },
    ui_builder::{UiBuilder, UiBuilderExt, UiRoot},
    ui_style::{AnimatedVals, SetWidthExt, StyleBuilder},
    widgets::prelude::*,
    SickleUiPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sickle UI -  Hundred Themes".into(),
                present_mode: bevy::window::PresentMode::Immediate,
                mode: bevy::window::WindowMode::BorderlessFullscreen,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            FrameTimeDiagnosticsPlugin,
            // Adds a system that prints diagnostics to the console
            LogDiagnosticsPlugin::default(),
        ))
        .add_plugins(SickleUiPlugin)
        .init_resource::<IconCache>()
        .add_plugins(ComponentThemePlugin::<ThemeTestBox>::new())
        .add_systems(Startup, setup)
        .add_systems(Update, (tick_dynamic_style_stopwatch, toggle_theme_data))
        .run();
}

fn tick_dynamic_style_stopwatch(time: Res<Time<Real>>, mut q_stopwatch: Query<&mut TestStopwatch>) {
    let mut stopwatch = q_stopwatch.single_mut();
    stopwatch.0.tick(time.delta());
}

fn toggle_theme_data(
    mut theme_data: ResMut<ThemeData>,
    mut q_stopwatch: Query<&mut TestStopwatch>,
) {
    let mut stopwatch = q_stopwatch.single_mut();

    if stopwatch.0.elapsed_secs() > 0.2 {
        stopwatch.0.reset();

        if theme_data.active_scheme.is_dark() {
            theme_data.active_scheme = Scheme::Light(Contrast::Standard);
        } else {
            theme_data.active_scheme = Scheme::Dark(Contrast::Standard);
        }
    }
}

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
pub struct TestStopwatch(Stopwatch);

#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
struct IconCache(Vec<Handle<Image>>);

#[derive(Component, Clone, UiContext)]
pub struct ThemeTestBox;

impl DefaultTheme for ThemeTestBox {
    fn default_theme() -> Option<Theme<Self>> {
        ThemeTestBox::base_theme().into()
    }
}

const BOX_SIZE: f32 = 100.;
const COLOR_B: Color = Color::rgb(0.1, 0.1, 0.1);
const COLOR_I: Color = Color::rgb(0.2, 0.2, 0.2);
const COLOR_1: Color = Color::rgb(0.25, 0.25, 0.25);
const COLOR_2: Color = Color::rgb(0.3, 0.3, 0.3);
const COLOR_3: Color = Color::rgb(0.35, 0.35, 0.35);
const COLOR_4: Color = Color::rgb(0.38, 0.38, 0.38);
const COLOR_5: Color = Color::rgb(0.41, 0.41, 0.41);
const COLOR_6: Color = Color::rgb(0.43, 0.43, 0.43);

impl ThemeTestBox {
    fn base_theme() -> Theme<ThemeTestBox> {
        let base_style = PseudoTheme::deferred(None, ThemeTestBox::base_style);
        let enabled_style =
            PseudoTheme::deferred(vec![PseudoState::Enabled], ThemeTestBox::enabled_style);
        let disabled_style =
            PseudoTheme::deferred(vec![PseudoState::Disabled], ThemeTestBox::disabled_style);
        let selected_style =
            PseudoTheme::deferred(vec![PseudoState::Selected], ThemeTestBox::selected_style);
        Theme::<ThemeTestBox>::new(vec![
            base_style,
            enabled_style,
            disabled_style,
            selected_style,
        ])
    }

    fn second_theme() -> Theme<ThemeTestBox> {
        let base_style = PseudoTheme::deferred(None, ThemeTestBox::second_base_style);
        let enabled_style =
            PseudoTheme::deferred(vec![PseudoState::Enabled], ThemeTestBox::enabled_style);
        let disabled_style =
            PseudoTheme::deferred(vec![PseudoState::Disabled], ThemeTestBox::disabled_style);
        let selected_style =
            PseudoTheme::deferred(vec![PseudoState::Selected], ThemeTestBox::selected_style);
        Theme::<ThemeTestBox>::new(vec![
            base_style,
            enabled_style,
            disabled_style,
            selected_style,
        ])
    }

    fn third_theme() -> Theme<ThemeTestBox> {
        let base_style = PseudoTheme::deferred(None, ThemeTestBox::third_base_style);
        let enabled_style =
            PseudoTheme::deferred(vec![PseudoState::Enabled], ThemeTestBox::enabled_style);
        let disabled_style =
            PseudoTheme::deferred(vec![PseudoState::Disabled], ThemeTestBox::disabled_style);
        let selected_style =
            PseudoTheme::deferred(vec![PseudoState::Selected], ThemeTestBox::selected_style);
        Theme::<ThemeTestBox>::new(vec![
            base_style,
            enabled_style,
            disabled_style,
            selected_style,
        ])
    }

    fn base_style(builder: &mut StyleBuilder, _: &ThemeData) {
        builder
            .width(Val::Px(BOX_SIZE))
            .height(Val::Px(BOX_SIZE))
            .flex_wrap(FlexWrap::Wrap)
            .animated()
            .background_color(AnimatedVals {
                idle: COLOR_1,
                enter_from: COLOR_B.into(),
                idle_alt: COLOR_I.into(),
                ..default()
            })
            .enter(2., Ease::OutExpo, 0.)
            .idle(0.5, Ease::InOutExpo, 0., 0., AnimationLoop::Continous);
    }

    fn enabled_style(builder: &mut StyleBuilder, _: &ThemeData) {
        builder
            .width(Val::Px(BOX_SIZE / 2.))
            .height(Val::Px(BOX_SIZE / 2.))
            .animated()
            .background_color(AnimatedVals {
                idle: COLOR_2,
                enter_from: COLOR_B.into(),
                ..default()
            })
            .enter(2., Ease::OutExpo, 0.125);
    }

    fn disabled_style(builder: &mut StyleBuilder, _: &ThemeData) {
        builder
            .width(Val::Px(BOX_SIZE / 2.))
            .height(Val::Px(BOX_SIZE / 2.))
            .animated()
            .background_color(AnimatedVals {
                idle: COLOR_3,
                enter_from: COLOR_B.into(),
                ..default()
            })
            .enter(2., Ease::OutExpo, 0.25);
    }

    fn selected_style(builder: &mut StyleBuilder, _: &ThemeData) {
        builder
            .width(Val::Px(BOX_SIZE / 2.))
            .height(Val::Px(BOX_SIZE / 2.))
            .animated()
            .background_color(AnimatedVals {
                idle: COLOR_4,
                enter_from: COLOR_B.into(),
                ..default()
            })
            .enter(2., Ease::OutExpo, 0.375);
    }

    fn second_base_style(builder: &mut StyleBuilder, _: &ThemeData) {
        builder
            .animated()
            .background_color(AnimatedVals {
                idle: COLOR_5,
                enter_from: COLOR_B.into(),
                ..default()
            })
            .enter(2., Ease::OutExpo, 0.);
    }

    fn third_base_style(builder: &mut StyleBuilder, _: &ThemeData) {
        builder
            .animated()
            .background_color(AnimatedVals {
                idle: COLOR_6,
                enter_from: COLOR_B.into(),
                ..default()
            })
            .enter(2., Ease::OutExpo, 0.);
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

    commands.ui_builder(UiRoot).container(
        (
            TestStopwatch(Stopwatch::new()),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    flex_wrap: FlexWrap::Wrap,
                    flex_grow: 1.,
                    justify_self: JustifySelf::Stretch,
                    align_self: AlignSelf::Stretch,
                    ..default()
                },
                background_color: Color::DARK_GRAY.into(),
                ..default()
            },
            TargetCamera(main_camera),
        ),
        |container| {
            // container.spawn((NodeBundle::default(), ThemeTestBox));
            for i in 0..14 {
                let mut row = container.row(|row| {
                    let mut column = row.column(|column| {
                        column.row(|inner_row| {
                            for _ in 0..12 {
                                spawn_test_content(inner_row);
                            }
                        });
                    });
                    column.style().width(Val::Percent(50.));

                    if i % 3 == 0 {
                        column.insert(ThemeTestBox::third_theme());
                    }

                    row.column(|column| {
                        column.row(|inner_row| {
                            for _ in 0..12 {
                                spawn_test_content(inner_row);
                            }
                        });
                    })
                    .style()
                    .width(Val::Percent(50.));
                });

                if i % 2 == 1 {
                    row.insert(ThemeTestBox::second_theme());
                }
            }
        },
    );
}

fn spawn_test_content(container: &mut UiBuilder<'_, '_, '_, Entity>) {
    container.container((NodeBundle::default(), ThemeTestBox), |main_box| {
        main_box.spawn((
            NodeBundle::default(),
            ThemeTestBox,
            PseudoStates::from(vec![PseudoState::Enabled]),
        ));
        main_box.spawn((
            NodeBundle::default(),
            ThemeTestBox,
            PseudoStates::from(vec![PseudoState::Disabled]),
        ));
        main_box.spawn((
            NodeBundle::default(),
            ThemeTestBox,
            PseudoStates::from(vec![PseudoState::Selected]),
        ));
    });
}
