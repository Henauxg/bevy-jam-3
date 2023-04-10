use std::f32::consts::PI;

use assets::GameAssets;
use bevy::{
    app::AppExit,
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::common_conditions::input_toggle_active,
    prelude::{
        default, in_state, shape, Added, App, Assets, BuildChildren, Color, Commands, Component,
        CoreSchedule, EventReader, EventWriter, Input, IntoSystemAppConfig, IntoSystemConfig,
        KeyCode, Mesh, Name, NodeBundle, OnEnter, OnUpdate, PbrBundle, PluginGroup, Quat, Query,
        Res, ResMut, StandardMaterial, States, TextBundle, Transform, Vec3, Visibility, With,
    },
    text::{Text, TextSection, TextStyle},
    ui::{AlignItems, JustifyContent, PositionType, Size, Style, UiRect, Val},
    window::{PresentMode, Window, WindowCloseRequested, WindowPlugin},
    DefaultPlugins,
};

use bevy_mod_picking::{DefaultHighlighting, DefaultPickingPlugins};
use bevy_tweening::TweeningPlugin;
use camera::{setup_camera, CustomOrbitCameraPlugin};

use data::{level_1, level_2, level_3, test_level_data, LevelData};
use debug::display_stats_ui;
use grass::setup_grass;
use logic::{
    climber::{update_climbers, ClimberEvent},
    face::Face,
    handle_win_pylon_pick_events,
    level::{
        climber_event_handler, level_event_handler, spawn_level, GameLevels, LevelEvent, LevelName,
    },
    pillar::Pillar,
    rod::handle_movable_rod_picking_events,
    Pylon, TilePosition, TileType,
};
use smooth_bevy_cameras::LookTransformPlugin;

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
#[cfg(debug_assertions)]
use debug::EguiInputBlockerPlugin;
use warbler_grass::warblers_plugin::WarblersPlugin;

mod assets;
mod camera;
mod data;
mod grass;
mod logic;

#[cfg(debug_assertions)]
mod debug;

#[derive(Default, Clone, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Playing,
    Lost,
    Won,
}

// THEMES

// Pillar material 193, 109, 0, 255
// Front dir light WHITE

// Autumn
// Ambient 255, 68, 0, 255  Brightness 0.2
// Clear color 109, 241, 255, 255
// Grass MC 149, 45, 0, 255
// Grass BC 34, 6, 6, 255
// Ground 79, 30, 0, 255

// Fushia

pub const CAMERA_CLEAR_COLOR: Color = Color::rgb(0.25, 0.55, 0.92); // 0, 0, 28, 255

const WINDOW_TITLE: &str = "Bevy-jam-3";

pub fn exit_on_window_close_system(
    mut app_exit_events: EventWriter<AppExit>,
    mut window_close_requested_events: EventReader<WindowCloseRequested>,
) {
    if !window_close_requested_events.is_empty() {
        app_exit_events.send(AppExit);
        window_close_requested_events.clear();
    }
}

fn skip_level(mut level_events: EventWriter<LevelEvent>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::N) {
        level_events.send(LevelEvent::LoadNext);
    }
}

fn handle_restart_key(
    mut level_events: EventWriter<LevelEvent>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        level_events.send(LevelEvent::Reload);
    }
}

#[derive(Component, Clone, Debug)]
struct LevelNameUI;

fn handle_new_levels(
    mut new_level: Query<&LevelName, Added<LevelName>>,
    mut level_name_ui: Query<&mut Text, With<LevelNameUI>>,
) {
    for loaded_level in new_level.iter_mut() {
        let mut text = level_name_ui.single_mut();
        text.sections.first_mut().unwrap().value = loaded_level.0.clone();
    }
}

#[derive(Component, Clone, Debug)]
struct GameOverText;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<GameAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_levels: ResMut<GameLevels>,
) {
    // sky
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Box::default())),
    //     material: materials.add(StandardMaterial {
    //         base_color: Color::hex("888888").unwrap(),
    //         unlit: true,
    //         cull_mode: None,
    //         ..default()
    //     }),
    //     transform: Transform::from_scale(Vec3::splat(1_000_000.0)),
    //     ..default()
    // });

    let text_style = TextStyle {
        font: assets.font.clone(),
        font_size: 45.0,
        color: Color::WHITE,
    };
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_sections([TextSection::new(
                    "A climber has fallen. Press Space to restart.",
                    text_style.clone(),
                )])
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    position: UiRect {
                        bottom: Val::Px(85.0),
                        ..default()
                    },
                    ..default()
                }),
                GameOverText,
            ));
        });
    let text_style = TextStyle {
        font: assets.font.clone(),
        font_size: 30.0,
        color: Color::WHITE,
    };
    commands.spawn((
        TextBundle::from_sections([TextSection::new("LevelName", text_style)]).with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
        LevelNameUI,
    ));

    // Ground TODO : move to level specific
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Circle::new(20.))),
            material: materials.add(StandardMaterial {
                base_color: Color::DARK_GREEN,
                // unlit: true,
                // cull_mode: None,
                ..default()
            }),
            transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::X, -1. * PI / 2.)),
            ..default()
        },
        Name::new("Ground"),
    ));

    commands.insert_resource(DefaultHighlighting {
        hovered: assets.movable_rod_highlight_mat.clone(),
        pressed: assets.movable_rod_mat.clone(),
        selected: assets.climber_mat.clone(),
    });

    // Spawn first level
    game_levels.current_level_entity = Some(spawn_level(
        &game_levels.get_current_level_data(),
        commands,
        meshes,
        assets,
        // materials,
    ));
}

fn hide_gameover_ui(mut game_over_ui: Query<&mut Visibility, With<GameOverText>>) {
    let mut ui = game_over_ui.single_mut();
    *ui = Visibility::Hidden;
}

fn show_gameover_ui(mut game_over_ui: Query<&mut Visibility, With<GameOverText>>) {
    let mut ui = game_over_ui.single_mut();
    *ui = Visibility::Visible;
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: WINDOW_TITLE.into(),
            resolution: (800., 600.).into(),
            present_mode: PresentMode::AutoVsync,
            // Tells wasm to resize the window according to the available canvas
            fit_canvas_to_parent: true,
            // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }))
    .add_plugin(TweeningPlugin)
    .add_plugin(LookTransformPlugin)
    .add_plugin(CustomOrbitCameraPlugin)
    .add_plugins(DefaultPickingPlugins)
    .add_plugin(WarblersPlugin);

    let mut level_builders: Vec<fn() -> LevelData> = vec![level_1, level_2, level_3];
    #[cfg(debug_assertions)]
    {
        level_builders.push(test_level_data);
    }
    app.init_resource::<GameAssets>()
        .insert_resource(GameLevels::new(level_builders));

    app.add_state::<GameState>()
        .add_event::<LevelEvent>()
        .add_event::<ClimberEvent>();

    app.add_startup_system(setup_camera)
        .add_startup_system(setup_scene)
        .add_startup_system(setup_grass);

    app.add_system(level_event_handler)
        .add_system(handle_restart_key)
        .add_system(handle_new_levels)
        .add_system(exit_on_window_close_system)
        .add_system(climber_event_handler);
    app.add_system(hide_gameover_ui.in_schedule(OnEnter(GameState::Playing)))
        .add_system(show_gameover_ui.in_schedule(OnEnter(GameState::Lost)))
        .add_system(handle_movable_rod_picking_events.in_set(OnUpdate(GameState::Playing)))
        .add_system(
            update_climbers
                .in_schedule(CoreSchedule::FixedUpdate)
                .run_if(in_state(GameState::Playing)),
        )
        .add_system(handle_win_pylon_pick_events.in_set(OnUpdate(GameState::Won)));

    #[cfg(debug_assertions)]
    {
        app.add_plugin(WorldInspectorPlugin::new().run_if(input_toggle_active(true, KeyCode::F2)))
            .add_plugin(EguiInputBlockerPlugin)
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_system(display_stats_ui.run_if(input_toggle_active(true, KeyCode::F3)))
            .add_system(skip_level);
    }

    app.run();
}
