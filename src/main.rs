use std::{collections::HashMap, time::Duration};

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
        default, info, shape, App, Assets, BuildChildren, Bundle, Color, Commands, Component,
        CoreSchedule, DirectionalLight, DirectionalLightBundle, Entity, EulerRot, EventReader,
        EventWriter, IntoSystemAppConfig, IntoSystemConfig, KeyCode, Mesh, Name, PbrBundle,
        PluginGroup, Quat, Query, Res, ResMut, SpatialBundle, Transform, Vec3, With,
    },
    window::{PresentMode, Window, WindowCloseRequested, WindowPlugin},
    DefaultPlugins,
};

use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle, PickingEvent, SelectionEvent};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween, TweeningPlugin};
use camera::{camera_input_map, setup_camera};

use climber::{spawn_climber, update_climbers, ClimberPosition};
use debug::display_stats_ui;
use level::{test_level_data, ClimberDirection, FaceDirection, FaceSize, PillarData, TileDataType};
use smooth_bevy_cameras::{controllers::orbit::OrbitCameraPlugin, LookTransformPlugin};

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
#[cfg(debug_assertions)]
use debug::EguiInputBlockerPlugin;

mod assets;
mod camera;
mod climber;
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

#[derive(Clone, Copy, Debug)]
pub struct TilePosition {
    i: u16,
    j: u16,
}
#[derive(Component, Clone, Debug)]
pub struct MovableRod {
    pub face: Entity,
    pub opposite_face: Entity,
    pub position: TilePosition,
}
impl MovableRod {
    fn swap_face(&mut self) {
        let tmp_face = self.face;
        self.face = self.opposite_face;
        self.opposite_face = tmp_face;
    }
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
    mut rods_animators: Query<(&Transform, &mut Animator<Transform>, &mut MovableRod)>,
    mut faces: Query<&mut Face>,
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
                if let Ok((rod_transform, mut rod_animator, mut rod)) =
                    rods_animators.get_mut(*entity)
                {
                    // TODO Add a criteria here

                    // Immediately set void for this face
                    let mut face = faces
                        .get_mut(rod.face)
                        .expect("Rod does not appear to have a Face reference");
                    face.remove_tile_at(rod.position);

                    let mut opposite_face = faces
                        .get_mut(rod.opposite_face)
                        .expect("Rod does not appear to have a Face reference");
                    // TODO set MovingRod on the other face after a delay (animation duration / 2)
                    opposite_face.set_tile_at(rod.position, TileType::MovableRod);

                    rod.swap_face();

                    if rod_animator.tweenable().progress() >= 1.0 {
                        // TODO Use another cirteria
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

fn spawn_movable_rod(
    commands: &mut Commands,
    assets: &Res<GameAssets>,
    face: Entity,
    opposite_face: Entity,
    tile_pos: TilePosition,
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
            MovableRod {
                face,
                position: tile_pos,
                opposite_face,
            },
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
                illuminance: 50000.,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -2.5, 0.5, 0.),
                scale: Vec3::new(3., 3., 1.), // TODO Fix: Smaller hides some shadows
                ..default()
            },
            // The default cascade config is designed to handle large scenes.
            // As this example has a much smaller world, we can tighten the shadow
            // bounds for better visual quality.
            cascade_shadow_config: CascadeShadowConfigBuilder {
                first_cascade_far_bound: 8.0,
                maximum_distance: 40.0,
                ..default()
            }
            .into(),
            ..default()
        })
        .id();
    let dir_light_back = commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                // Configure the projection to better fit the scene
                shadows_enabled: false,
                color: Color::ALICE_BLUE,
                illuminance: 5000.,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_euler(EulerRot::XYZ, 8.5, 3.5, 0.),
                scale: Vec3::new(3., 3., 1.), // TODO Fix: Smaller hides some shadows
                ..default()
            },
            // The default cascade config is designed to handle large scenes.
            // As this example has a much smaller world, we can tighten the shadow
            // bounds for better visual quality.
            cascade_shadow_config: CascadeShadowConfigBuilder {
                first_cascade_far_bound: 8.0,
                maximum_distance: 40.0,
                ..default()
            }
            .into(),
            ..default()
        })
        .id();
    commands.entity(level_entity).add_child(dir_light);
    commands.entity(level_entity).add_child(dir_light_back);

    for pillar in level_data.pillars {
        let pillar_entity = spawn_pillar(&mut commands, &mut meshes, &assets, &pillar);
        commands.entity(level_entity).add_child(pillar_entity);

        let face_entities = HashMap::from([
            (FaceDirection::West, commands.spawn_empty().id()),
            (FaceDirection::North, commands.spawn_empty().id()),
            (FaceDirection::East, commands.spawn_empty().id()),
            (FaceDirection::South, commands.spawn_empty().id()),
        ]);

        for (face_direction, face) in pillar.faces {
            let &face_entity = face_entities.get(&face_direction).unwrap();
            let opposite_face_entity = face_entities
                .get(&FaceDirection::get_opposite(&face_direction))
                .unwrap();
            let factor = match face_direction {
                level::FaceDirection::West => -1.,
                level::FaceDirection::North => 1., // TODO North south
                level::FaceDirection::East => 1.,
                level::FaceDirection::South => -1., // TODO North south
            };

            // move on x axis for west/east
            // move on z axis for north/south

            let col = vec![TileType::Void; pillar.h as usize];
            let mut face_tiles = vec![col; pillar.w as usize];

            for tile in face.tiles {
                let tile_entity = match tile.kind {
                    // Relative to pillar position.
                    TileDataType::StaticRod => {
                        face_tiles[tile.i as usize][tile.j as usize] = TileType::StaticRod;
                        spawn_static_rod(
                            &mut commands,
                            &assets,
                            factor * (HALF_STATIC_ROD_LENGTH + pillar.w as f32 / 2.), // TODO North south
                            tile.j as f32 * GAME_UNIT + HALF_GAME_UNIT - pillar.h as f32 / 2., // TODO + HALF_PILLAR_WIDTH ? Where is the origin of the 3d mesh ?
                            tile.i as f32 * GAME_UNIT - pillar.w as f32 / 2. + HALF_GAME_UNIT,
                        )
                    }
                    TileDataType::MovableRod => {
                        face_tiles[tile.i as usize][tile.j as usize] = TileType::MovableRod;
                        spawn_movable_rod(
                            &mut commands,
                            &assets,
                            face_entity,
                            *opposite_face_entity,
                            TilePosition {
                                i: tile.i,
                                j: tile.j,
                            },
                            (factor * MOVABLE_ROD_MOVEMENT_AMPLITUDE) / 2., // TODO North south
                            tile.j as f32 * GAME_UNIT + HALF_GAME_UNIT - pillar.h as f32 / 2.,
                            tile.i as f32 * GAME_UNIT - pillar.w as f32 / 2. + HALF_GAME_UNIT,
                        )
                    }
                };
                commands.entity(pillar_entity).add_child(tile_entity);
            }

            // let face_entity = spawn_face(
            //     &mut commands,
            //     Vec3::new(
            //         pillar.x + factor * pillar.w as f32 / 2.,
            //         0.,
            //         pillar.z - pillar.w as f32 / 2., // TODO North south
            //     ),
            //     face.direction,
            //     FaceSize {
            //         w: pillar.w,
            //         h: pillar.h,
            //     },
            //     face_tiles,
            // );
            commands.entity(face_entity).insert(Face {
                origin: Vec3::new(
                    pillar.x + factor * pillar.w as f32 / 2.,
                    0.,
                    pillar.z - pillar.w as f32 / 2., // TODO North south
                ),
                direction: face_direction,
                size: FaceSize {
                    w: pillar.w,
                    h: pillar.h,
                },
                tiles: face_tiles,
            });
            commands.entity(pillar_entity).add_child(face_entity);

            for climber in face.climbers {
                let climber_entity = spawn_climber(
                    &mut commands,
                    &assets,
                    face_entity,
                    &climber,
                    factor * (HALF_GAME_UNIT + pillar.w as f32 / 2.),
                    climber.tile_j as f32 * GAME_UNIT
                        + HALF_GAME_UNIT
                        + CLIMBER_RADIUS * 1.2
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

#[derive(Component, Clone, Debug)]
pub struct Face {
    origin: Vec3,
    direction: FaceDirection,
    size: FaceSize,
    tiles: Vec<Vec<TileType>>,
}

impl Face {
    fn has_ground_on_tile(&self, pos: &ClimberPosition) -> bool {
        if pos.i >= self.size.w || pos.j >= self.size.h {
            return false;
        }
        self.tiles[pos.j as usize][pos.i as usize] != TileType::Void
    }

    fn get_pos_from_tile(&self, pos: &ClimberPosition) -> Vec3 {
        match self.direction {
            FaceDirection::West | FaceDirection::East => Vec3::new(
                self.origin.x,
                self.origin.y + pos.j as f32 * GAME_UNIT,
                self.origin.z + pos.i as f32 * GAME_UNIT,
            ),
            FaceDirection::North | FaceDirection::South => Vec3::new(
                self.origin.x + pos.i as f32 * GAME_UNIT,
                self.origin.y + pos.j as f32 * GAME_UNIT,
                self.origin.z,
            ),
        }
    }

    // Can return an invalid tile
    fn get_next_tile(
        &self,
        tile: &ClimberPosition,
        direction: &ClimberDirection,
    ) -> ClimberPosition {
        let tile_i = match direction {
            ClimberDirection::Increasing => tile.i + 1,
            ClimberDirection::Decreasing => tile.i - 1,
        };
        ClimberPosition {
            face: tile.face,
            i: tile_i,
            j: tile.j + 1,
        }
    }

    // No input checks
    fn remove_tile_at(&mut self, pos: TilePosition) {
        self.tiles[pos.j as usize][pos.i as usize] = TileType::Void;
    }

    fn set_tile_at(&mut self, pos: TilePosition, tile_type: TileType) {
        self.tiles[pos.j as usize][pos.i as usize] = tile_type;
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
