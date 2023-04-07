use std::{collections::HashMap, f32::consts::PI, time::Duration};

use assets::{
    GameAssets, CLIMBER_LEVITATE_DISTANCE, CLIMBER_RADIUS, HALF_ROD_WIDTH, HALF_TILE_SIZE,
    HALF_VISIBLE_ROD_LENGTH, MOVABLE_ROD_MOVEMENT_AMPLITUDE, TILE_SIZE,
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
        PluginGroup, Quat, Query, Res, ResMut, SpatialBundle, StandardMaterial, Transform, Vec2,
        Vec3,
    },
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    ui::{FocusPolicy, Interaction},
    window::{PresentMode, Window, WindowCloseRequested, WindowPlugin},
    DefaultPlugins,
};

use bevy_mod_picking::{
    highlight::Highlight, DefaultHighlighting, DefaultPickingPlugins, Hover, PickableMesh,
    PickingEvent, SelectionEvent,
};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween, TweeningPlugin};
use camera::{camera_input_map, setup_camera};

use climber::{spawn_climber, update_climbers, ClimberPosition};
use debug::display_stats_ui;
use level::{test_level_data, ClimberDirection, FaceDirection, FaceSize, PillarData, TileDataType};
use rand::{distributions::Uniform, prelude::Distribution};
use smooth_bevy_cameras::{controllers::orbit::OrbitCameraPlugin, LookTransformPlugin};

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
#[cfg(debug_assertions)]
use debug::EguiInputBlockerPlugin;
use warbler_grass::{
    prelude::{Grass, WarblersExplicitBundle},
    warblers_plugin::WarblersPlugin,
    GrassConfiguration,
};

mod assets;
mod camera;
mod climber;
mod level;

#[cfg(debug_assertions)]
mod debug;

pub const CAMERA_CLEAR_COLOR: Color = Color::rgb(0.25, 0.55, 0.92);

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
            // PickableBundle::default()
            Highlight::default(),
            Hover::default(),
            FocusPolicy::Block,
            Interaction::default(),
            PickableMesh::default(),
            Animator::new(tween),
            Name::from("Movable Rod"),
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
        .insert(Name::from("Static Rod"))
        .id()
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<GameAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    // Ground
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Circle::new(20.))),
        material: materials.add(StandardMaterial {
            base_color: Color::DARK_GREEN,
            // unlit: true,
            // cull_mode: None,
            ..default()
        }),
        transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::X, -1. * PI / 2.)),
        ..default()
    });

    let level_data = test_level_data();
    let level_entity = commands.spawn(Levelbundle::new(&level_data.name)).id();

    commands.insert_resource(DefaultHighlighting {
        hovered: assets.movable_rod_highlight_mat.clone(),
        pressed: assets.movable_rod_mat.clone(),
        selected: assets.climber_mat.clone(),
    });

    // Ambient light
    commands.insert_resource(level_data.ambient_light.clone());

    // The default cascade config is designed to handle large scenes.
    // As this example has a much smaller world, we can tighten the shadow
    // bounds for better visual quality.
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 8.0,
        maximum_distance: 40.0,
        ..default()
    }
    .build();
    // Dir light
    let dir_light = commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                color: level_data.dir_light_color,
                illuminance: 50000.,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -2.5, 0.5, 0.),
                scale: Vec3::new(3., 3., 1.),
                ..default()
            },
            cascade_shadow_config: cascade_shadow_config.clone(),
            ..default()
        })
        .insert(Name::from("Front directional light"))
        .id();
    let dir_light_back = commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: false,
                color: Color::ALICE_BLUE,
                illuminance: 5000.,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_euler(EulerRot::XYZ, 8.5, 3.5, 0.),
                scale: Vec3::new(3., 3., 1.),
                ..default()
            },
            cascade_shadow_config: cascade_shadow_config.clone(),
            ..default()
        })
        .insert(Name::from("Back directional light"))
        .id();
    commands.entity(level_entity).add_child(dir_light);
    commands.entity(level_entity).add_child(dir_light_back);

    for pillar in level_data.pillars {
        let pillar_entity = spawn_pillar(&mut commands, &mut meshes, &assets, &pillar);
        commands.entity(level_entity).add_child(pillar_entity);

        let pillar_half_width = pillar.w as f32 * TILE_SIZE / 2.;
        let pillar_half_height = pillar.h as f32 * TILE_SIZE / 2.;

        let face_entities = HashMap::from([
            (
                FaceDirection::West,
                commands.spawn(Name::from("West face")).id(),
            ),
            (
                FaceDirection::North,
                commands.spawn(Name::from("North face")).id(),
            ),
            (
                FaceDirection::East,
                commands.spawn(Name::from("East face")).id(),
            ),
            (
                FaceDirection::South,
                commands.spawn(Name::from("South face")).id(),
            ),
        ]);

        for (face_direction, face) in pillar.faces {
            let &face_entity = face_entities.get(&face_direction).unwrap();
            let opposite_face_entity = face_entities.get(&face_direction.get_opposite()).unwrap();
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
                            factor * (HALF_VISIBLE_ROD_LENGTH + pillar_half_width), // TODO North south
                            tile.j as f32 * TILE_SIZE + HALF_TILE_SIZE - pillar_half_height, // TODO + HALF_PILLAR_WIDTH ? Where is the origin of the 3d mesh ?
                            tile.i as f32 * TILE_SIZE + HALF_TILE_SIZE - pillar_half_width,
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
                            tile.j as f32 * TILE_SIZE + HALF_TILE_SIZE - pillar_half_height,
                            tile.i as f32 * TILE_SIZE + HALF_TILE_SIZE - pillar_half_width,
                        )
                    }
                };
                commands.entity(pillar_entity).add_child(tile_entity);
            }

            commands.entity(face_entity).insert(Face {
                origin: Vec3::new(
                    pillar.x + factor * pillar_half_width,
                    0.,
                    pillar.z - pillar_half_width, // TODO North south
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
                    factor * (HALF_VISIBLE_ROD_LENGTH + pillar_half_width),
                    climber.tile_j as f32 * TILE_SIZE
                        + HALF_TILE_SIZE
                        + HALF_ROD_WIDTH
                        + CLIMBER_RADIUS
                        + CLIMBER_LEVITATE_DISTANCE,
                    climber.tile_i as f32 * TILE_SIZE - pillar_half_width + HALF_VISIBLE_ROD_LENGTH,
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
                        pillar_data.w as f32 * TILE_SIZE,
                        pillar_data.h as f32 * TILE_SIZE,
                        pillar_data.w as f32 * TILE_SIZE,
                    )
                    .into(),
                ),
                material: assets.pillar_mat.clone(),
                transform: Transform::from_translation(Vec3::new(
                    pillar_data.x,
                    pillar_data.h as f32 * TILE_SIZE / 2.,
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
            Name::from("Pillar"),
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
        self.tiles[pos.i as usize][pos.j as usize] != TileType::Void
    }

    fn climber_get_pos_from_tile(&self, pos: &ClimberPosition) -> Vec3 {
        let factor = match self.direction {
            FaceDirection::West | FaceDirection::South => -1.,
            FaceDirection::North | FaceDirection::East => 1.,
        };
        let y = self.origin.y
            + pos.j as f32 * TILE_SIZE
            + TILE_SIZE
            + CLIMBER_RADIUS
            + CLIMBER_LEVITATE_DISTANCE;
        let horizontal_delta = pos.i as f32 * TILE_SIZE + HALF_TILE_SIZE;
        match self.direction {
            FaceDirection::West | FaceDirection::East => Vec3::new(
                self.origin.x + factor * HALF_VISIBLE_ROD_LENGTH,
                y,
                self.origin.z + horizontal_delta,
            ),
            FaceDirection::North | FaceDirection::South => Vec3::new(
                self.origin.x + horizontal_delta,
                y,
                self.origin.z + factor * HALF_VISIBLE_ROD_LENGTH,
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

fn custom_grass_mesh() -> Mesh {
    let mut grass_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    grass_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0., 0., 0.],
            [0.25, 0., 0.],
            [0.125, 0., 0.2],
            [0.125, 1.0, 0.075],
        ],
    );
    grass_mesh.set_indices(Some(Indices::U32(vec![1, 0, 3, 2, 1, 3, 0, 2, 3])));
    grass_mesh
}

fn setup_grass(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(GrassConfiguration {
        main_color: Color::rgb(0.2, 0.5, 0.0),
        bottom_color: Color::rgb(0.1, 0.1, 0.0),
        wind: Vec2::new(0.6, 0.6),
    });

    let grass_mesh = meshes.add(custom_grass_mesh());

    let base_count = 400;
    let per_radius_count = 75;
    let noise_range = Uniform::from(0.9..1.1);
    let mut rng = rand::thread_rng();

    let mut positions: Vec<Vec3> = Vec::new();
    for radius in 3..45 {
        let grass_blades_count = base_count + radius * per_radius_count;
        for i in 0..grass_blades_count {
            let f = i as f32 / grass_blades_count as f32;
            let dist = radius as f32 / 2.0;
            let noise: f32 = noise_range.sample(&mut rng);
            let x = (f * std::f32::consts::PI * 2.).cos() * dist * noise;
            let y = (f * std::f32::consts::PI * 2.).sin() * dist * noise;
            positions.push(Vec3::new(x, 0., y))
        }
    }

    commands.spawn(WarblersExplicitBundle {
        grass_mesh,
        grass: Grass {
            positions,
            height: 0.3,
        },
        ..default()
    });
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
    .add_plugins(DefaultPickingPlugins)
    .add_plugin(WarblersPlugin);

    app.init_resource::<GameAssets>();

    app.add_startup_system(setup_camera)
        .add_startup_system(setup_scene)
        .add_startup_system(setup_grass);

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
