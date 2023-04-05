use std::time::Duration;

use assets::{
    GameAssets, CLIMBER_RADIUS, HALF_PILLAR_WIDTH, HALF_ROD_WIDTH, PILLAR_HEIGHT, PILLAR_WIDTH,
};
use bevy::{
    app::AppExit,
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::common_conditions::input_toggle_active,
    pbr::CascadeShadowConfigBuilder,
    prelude::{
        default, info, shape, AmbientLight, App, Assets, BuildChildren, Bundle, Color, Commands,
        Component, CoreSchedule, DirectionalLight, DirectionalLightBundle, Entity, EulerRot,
        EventReader, EventWriter, IntoSystemAppConfig, IntoSystemConfig, KeyCode, Mesh, Name,
        PbrBundle, PluginGroup, Quat, Query, Res, ResMut, SpatialBundle, Transform, Vec3, With,
    },
    window::{PresentMode, Window, WindowCloseRequested, WindowPlugin},
    DefaultPlugins,
};

use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle, PickingEvent, SelectionEvent};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween, TweeningPlugin};
use camera::{camera_input_map, setup_camera};

use debug::display_stats_ui;
use smooth_bevy_cameras::{controllers::orbit::OrbitCameraPlugin, LookTransformPlugin};

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
#[cfg(debug_assertions)]
use debug::EguiInputBlockerPlugin;

mod assets;
mod camera;
#[cfg(debug_assertions)]
mod debug;

pub const CAMERA_CLEAR_COLOR: Color = Color::rgb(0.25, 0.55, 0.92);

pub const GAME_UNIT: f32 = 1.0;
pub const HALF_GAME_UNIT: f32 = GAME_UNIT / 2.;

const WINDOW_TITLE: &str = "Bevy-jam-3";

#[derive(Bundle, Default)]
pub struct Levelbundle {
    name: Name,
    spatial: SpatialBundle,
}

impl Levelbundle {
    pub fn new(name: &str) -> Self {
        Self {
            name: Name::from(name),
            ..default()
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct Rod {}

#[derive(Component, Clone, Debug)]
pub struct MovableRod {}

#[derive(Component, Clone, Debug)]
pub struct Climber {
    state: ClimberState,
}

pub fn exit_on_window_close_system(
    mut app_exit_events: EventWriter<AppExit>,
    mut window_close_requested_events: EventReader<WindowCloseRequested>,
) {
    if !window_close_requested_events.is_empty() {
        app_exit_events.send(AppExit);
        window_close_requested_events.clear();
    }
}

fn handle_picking_events(
    mut events: EventReader<PickingEvent>,
    mut rods_animators: Query<(&Transform, &mut Animator<Transform>), With<MovableRod>>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(SelectionEvent::JustSelected(entity)) => {
                info!("SelectionEvent JustSelected {:?}", entity);
            }
            PickingEvent::Selection(SelectionEvent::JustDeselected(entity)) => {
                info!("SelectionEvent JustDeselected {:?}", entity);
            }
            PickingEvent::Hover(_) => {}
            PickingEvent::Clicked(entity) => {
                if let Ok((rod_transform, mut rod_animator)) = rods_animators.get_mut(*entity) {
                    if rod_animator.tweenable().progress() >= 1.0 {
                        let tween = Tween::new(
                            EaseFunction::QuadraticInOut,
                            Duration::from_secs(1),
                            TransformPositionLens {
                                start: rod_transform.translation,
                                end: Vec3::new(
                                    -rod_transform.translation.x,
                                    rod_transform.translation.y,
                                    rod_transform.translation.z,
                                ),
                            },
                        );
                        rod_animator.set_tweenable(tween);
                    }
                    // TODO Could reverse it if interacting again while active
                }
            }
        }
    }
}

fn spawn_climber(
    commands: &mut Commands,
    assets: &Res<GameAssets>,
    x: f32,
    y: f32,
    z: f32,
) -> Entity {
    commands
        .spawn((PbrBundle {
            mesh: assets.climber_mesh.clone(),
            material: assets.climber_mat.clone(),
            transform: Transform::from_xyz(x, y, z),
            ..default()
        },))
        .insert(Climber {
            state: ClimberState::Waiting,
        })
        // .insert(PickableBundle::default())
        .id()
}

fn spawn_movable_rod(
    commands: &mut Commands,
    assets: &Res<GameAssets>,
    x: f32,
    y: f32,
    z: f32,
) -> Entity {
    // Dummy tween
    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(1),
        TransformPositionLens {
            start: Vec3::new(x, y, z),
            end: Vec3::new(x, y, z),
        },
    )
    .with_repeat_count(0);

    commands
        .spawn((
            PbrBundle {
                mesh: assets.movable_rod_mesh.clone(),
                material: assets.movable_rod_mat.clone(),
                transform: Transform::from_xyz(x, y, z),
                ..default()
            },
            Rod {},
            MovableRod {},
            PickableBundle::default(),
            Animator::new(tween),
        ))
        .id()
}

fn spawn_static_rod(commands: &mut Commands, assets: &Res<GameAssets>, y: f32, z: f32) -> Entity {
    commands
        .spawn((PbrBundle {
            mesh: assets.static_rod_mesh.clone(),
            material: assets.static_rod_mat.clone(),
            transform: Transform::from_xyz(0.0, y, z),
            ..default()
        },))
        .insert(Rod {})
        .id()
}

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, assets: Res<GameAssets>) {
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::ORANGE_RED,
        brightness: 0.2,
    });

    let level_entity = commands.spawn(Levelbundle::new("Test level")).id();

    let dir_light = commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                // Configure the projection to better fit the scene
                shadows_enabled: true,
                color: Color::WHITE,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.6, 0.7, 0.),
                scale: Vec3::new(3., 3., 1.), // TODO Fix: Smaller hides some shadows
                ..default()
            },
            // The default cascade config is designed to handle large scenes.
            // As this example has a much smaller world, we can tighten the shadow
            // bounds for better visual quality.
            cascade_shadow_config: CascadeShadowConfigBuilder {
                first_cascade_far_bound: 4.0,
                maximum_distance: 10.0,
                ..default()
            }
            .into(),
            ..default()
        })
        .id();

    let pillar = commands
        .spawn((PbrBundle {
            mesh: meshes.add(shape::Box::new(PILLAR_WIDTH, PILLAR_HEIGHT, PILLAR_WIDTH).into()),
            material: assets.pillar_mat.clone(),
            transform: Transform::from_xyz(0.0, PILLAR_HEIGHT / 2.0, 0.0),
            ..default()
        },))
        .id();

    let s_x_rod_1 = spawn_static_rod(&mut commands, &assets, HALF_ROD_WIDTH, -2.0);
    let s_x_rod_2 = spawn_static_rod(&mut commands, &assets, HALF_ROD_WIDTH + GAME_UNIT, -1.0);

    let x_rod_1 = spawn_movable_rod(&mut commands, &assets, -HALF_GAME_UNIT, 2.0, 2.0);
    let x_rod_2 = spawn_movable_rod(&mut commands, &assets, HALF_GAME_UNIT, 5.0, 1.0);

    let climber_1 = spawn_climber(
        &mut commands,
        &assets,
        HALF_PILLAR_WIDTH + HALF_GAME_UNIT,
        CLIMBER_RADIUS,
        -(HALF_PILLAR_WIDTH + HALF_GAME_UNIT),
    );
    let climber_2 = spawn_climber(
        &mut commands,
        &assets,
        -(HALF_PILLAR_WIDTH + HALF_GAME_UNIT),
        CLIMBER_RADIUS,
        -(HALF_PILLAR_WIDTH + HALF_GAME_UNIT),
    );

    commands
        .entity(level_entity)
        .add_child(dir_light)
        .add_child(pillar)
        .add_child(s_x_rod_1)
        .add_child(s_x_rod_2)
        .add_child(x_rod_1)
        .add_child(x_rod_2)
        .add_child(climber_1)
        .add_child(climber_2);
}

#[derive(Component, Clone, Debug)]
enum ClimberState {
    Waiting,
    Moving,
    Falling,
    Dead,
}

pub fn update_climbers(climbers: Query<(&Transform, &Climber, Entity)>) {
    for (transform, climber, entity) in &climbers {
        // info!("Climber update: {:?} in state {:?}", entity, climber.state);
        match climber.state {
            ClimberState::Waiting => {
                // If climber doesn't have a rod beneath him anymore : falling
                // If a rod can be reached: start moving to that rod
            }
            ClimberState::Moving => {
                // If the move is finished: waiting
            }
            ClimberState::Falling => {
                // If a rod is reached : waiting
                // If the void is reached : dead
            }
            ClimberState::Dead => (),
        }
    }
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
    .add_plugin(OrbitCameraPlugin::new(true))
    .add_plugins(DefaultPickingPlugins);

    app.init_resource::<GameAssets>();

    app.add_startup_system(setup_camera)
        .add_startup_system(setup_scene);

    app.add_system(camera_input_map)
        .add_system(handle_picking_events)
        .add_system(update_climbers.in_schedule(CoreSchedule::FixedUpdate))
        .add_system(exit_on_window_close_system);

    #[cfg(debug_assertions)]
    {
        app.add_plugin(WorldInspectorPlugin::new().run_if(input_toggle_active(true, KeyCode::F2)))
            .add_plugin(EguiInputBlockerPlugin)
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_system(display_stats_ui.run_if(input_toggle_active(true, KeyCode::F3)));
    }

    app.run();
}
