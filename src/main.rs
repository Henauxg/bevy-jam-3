use std::time::Duration;

use assets::{
    GameAssets, CLIMBER_RADIUS, HALF_ROD_WIDTH, HALF_STATIC_ROD_LENGTH,
    MOVABLE_ROD_MOVEMENT_AMPLITUDE,
};
use bevy::{
    app::AppExit,
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::common_conditions::input_toggle_active,
    pbr::CascadeShadowConfigBuilder,
    prelude::{
        default, error, info, shape, App, Assets, BuildChildren, Bundle, Color, Commands,
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
use level::{test_level_data, FaceSize, PillarData, TileDataType};
use smooth_bevy_cameras::{controllers::orbit::OrbitCameraPlugin, LookTransformPlugin};

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
#[cfg(debug_assertions)]
use debug::EguiInputBlockerPlugin;

mod assets;
mod camera;
mod level;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileType {
    Void,
    StaticRod,
    MovableRod,
}
// #[derive(Clone, Debug)]
// pub struct TileData {
//     pub kind: TileType,
// }

#[derive(Clone, Debug)]
pub struct PillarFace {
    // pub size: FaceSize,
    pub tiles: Vec<Vec<TileType>>,
}

#[derive(Component, Clone, Debug)]
pub struct Pillar {
    pub faces: Vec<PillarFace>,
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
    // pillar_data: &PillarData,
    // climber_data: &ClimberData,
    x: f32,
    y: f32,
    z: f32,
) -> Entity {
    // let climber_pos = pillar_data.position.clone();
    // climber_pos = match climber_data.start_position.face_index {

    // }
    // climber_pos.x= climber_data.start_position.face_index;
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

fn spawn_static_rod(
    commands: &mut Commands,
    assets: &Res<GameAssets>,
    x: f32,
    y: f32,
    z: f32,
) -> Entity {
    commands
        .spawn((PbrBundle {
            mesh: assets.static_rod_mesh.clone(),
            material: assets.static_rod_mat.clone(),
            transform: Transform::from_xyz(x, y, z), //.with_rotation(Quat::from_axis_angle(Vec3::Y, PI/2.)
            ..default()
        },))
        .insert(Rod {})
        .id()
}

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, assets: Res<GameAssets>) {
    let level_data = test_level_data();
    let level_entity = commands.spawn(Levelbundle::new(&level_data.name)).id();

    // Ambient light
    commands.insert_resource(level_data.ambient_light.clone());
    // Dir light
    let dir_light = commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                // Configure the projection to better fit the scene
                shadows_enabled: true,
                color: level_data.dir_light_color,
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
    commands.entity(level_entity).add_child(dir_light);

    for pillar in level_data.pillars {
        let pillar_entity = spawn_pillar(&mut commands, &mut meshes, &assets, &pillar);
        commands.entity(level_entity).add_child(pillar_entity);

        for face in pillar.faces {
            let factor = match face.direction {
                level::FaceDirection::West => -1.,
                level::FaceDirection::North => 1.,
                level::FaceDirection::East => 1.,
                level::FaceDirection::South => -1.,
            };
            // TODO North south
            // move on x axis for west/east
            // move on z axis for north/south

            for tile in face.tiles {
                let tile_entity = match tile.kind {
                    // Relative to pillar position.
                    TileDataType::StaticRod => spawn_static_rod(
                        &mut commands,
                        &assets,
                        factor * (HALF_STATIC_ROD_LENGTH + pillar.w as f32 / 2.),
                        tile.j as f32 * GAME_UNIT + HALF_GAME_UNIT - pillar.h as f32 / 2., // TODO + HALF_PILLAR_WIDTH ? Where is the origin of the 3d mesh ?
                        tile.i as f32 * GAME_UNIT - pillar.w as f32 / 2. + HALF_GAME_UNIT,
                    ),
                    TileDataType::MovableRod => spawn_movable_rod(
                        &mut commands,
                        &assets,
                        (factor * MOVABLE_ROD_MOVEMENT_AMPLITUDE) / 2.,
                        tile.j as f32 * GAME_UNIT + HALF_GAME_UNIT - pillar.h as f32 / 2.,
                        tile.i as f32 * GAME_UNIT - pillar.w as f32 / 2. + HALF_GAME_UNIT,
                    ),
                };
                commands.entity(pillar_entity).add_child(tile_entity);
            }
            for climber in face.climbers {
                let climber_entity = spawn_climber(
                    &mut commands,
                    &assets,
                    factor * (HALF_GAME_UNIT + pillar.w as f32 / 2.),
                    climber.tile_j as f32 * GAME_UNIT
                        + HALF_GAME_UNIT
                        + CLIMBER_RADIUS
                        + HALF_ROD_WIDTH,
                    climber.tile_i as f32 * GAME_UNIT - pillar.w as f32 / 2. + HALF_GAME_UNIT,
                );
                commands.entity(level_entity).add_child(climber_entity);
            }
        }
    }
}

fn spawn_pillar(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    assets: &Res<GameAssets>,
    pillar_data: &PillarData,
) -> Entity {
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(
                    shape::Box::new(
                        pillar_data.w.into(),
                        pillar_data.h.into(),
                        pillar_data.w.into(),
                    )
                    .into(),
                ),
                material: assets.pillar_mat.clone(),
                transform: Transform::from_translation(Vec3::new(
                    pillar_data.x,
                    pillar_data.h as f32 / 2.,
                    pillar_data.z,
                )),
                ..default()
            },
            Pillar {
                faces: vec![PillarFace {
                    // size: todo!(),
                    tiles: vec![],
                }],
            },
        ))
        .id()
}

#[derive(Clone, Debug)]
struct ClimberPosition {
    face: Entity,
    i: u16,
    j: u16,
}

#[derive(Component, Clone, Debug)]
enum ClimberState {
    Waiting {
        pos: ClimberPosition,
        next_pos: ClimberPosition,
    },
    Moving,
    Falling,
    Dead,
}

#[derive(Component, Clone, Debug)]
struct Face {
    size: FaceSize,
    tiles: Vec<Vec<TileType>>,
}

impl Face {
    fn has_ground_on_pos(&self, pos: ClimberPosition) -> bool {
        if pos.i >= self.size.w || pos.j >= self.size.h {
            return false;
        }
        self.tiles[pos.i as usize][pos.j as usize] != TileType::Void
    }
}

pub fn update_climbers(climbers: Query<(&Transform, &Climber, Entity)>, faces: Query<&Face>) {
    for (transform, climber, entity) in &climbers {
        match climber.state {
            ClimberState::Waiting { pos, next_pos } => {
                match faces.get(pos.face) {
                    Ok(face) => {
                        // If climber doesn't have a rod beneath him anymore : falling
                        if !face.has_ground_on_pos(pos) {
                            climber.state = ClimberState::Falling;
                        }
                        // If a rod can be reached: start moving to that rod
                        else if face.has_ground_on_pos(next_pos) {
                            climber.state = ClimberState::Moving;
                        }
                    }
                    Err(e) => error!("Waiting climber does not appear to have a Face reference."),
                }
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
