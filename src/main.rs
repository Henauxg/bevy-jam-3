use std::{collections::HashMap, f32::consts::PI};

use assets::{
    GameAssets, CLIMBER_LEVITATE_DISTANCE, CLIMBER_RADIUS, HALF_ROD_WIDTH, HALF_TILE_SIZE,
    HALF_VISIBLE_ROD_LENGTH, MOVABLE_ROD_MOVEMENT_AMPLITUDE, PYLON_HORIZONTAL_DELTA, TILE_SIZE,
};
use bevy::{
    app::AppExit,
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::common_conditions::input_toggle_active,
    pbr::CascadeShadowConfigBuilder,
    prelude::{
        default, shape, App, Assets, BuildChildren, Color, Commands, CoreSchedule,
        DirectionalLight, DirectionalLightBundle, Entity, EulerRot, EventReader, EventWriter,
        IntoSystemAppConfig, IntoSystemConfig, KeyCode, Mesh, Name, PbrBundle, PluginGroup, Quat,
        Res, ResMut, StandardMaterial, Transform, Vec3,
    },
    window::{PresentMode, Window, WindowCloseRequested, WindowPlugin},
    DefaultPlugins,
};

use bevy_mod_picking::{DefaultHighlighting, DefaultPickingPlugins};
use bevy_tweening::TweeningPlugin;
use camera::{camera_input_map, setup_camera};

use debug::display_stats_ui;
use grass::setup_grass;
use level::{test_level_data, FaceDirection, FaceSize, TileDataType};
use logic::{
    climber::{spawn_climber, update_climbers},
    face::Face,
    handle_picking_events,
    pillar::{spawn_pillar, Pillar},
    rod::{spawn_movable_rod, spawn_static_rod},
    Levelbundle, Pylon, TilePosition, TileType,
};
use smooth_bevy_cameras::{controllers::orbit::OrbitCameraPlugin, LookTransformPlugin};

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
#[cfg(debug_assertions)]
use debug::EguiInputBlockerPlugin;
use warbler_grass::warblers_plugin::WarblersPlugin;

mod assets;
mod camera;
mod grass;
mod level;
mod logic;

#[cfg(debug_assertions)]
mod debug;

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

        let mut unpowered_pylons: HashMap<FaceDirection, Vec<Entity>> = HashMap::from([
            (FaceDirection::East, vec![]),
            (FaceDirection::West, vec![]),
            (FaceDirection::North, vec![]),
            (FaceDirection::South, vec![]),
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

            let face_origin = Vec3::new(
                pillar.x + factor * pillar_half_width,
                0.,
                pillar.z - pillar_half_width, // TODO North south
            );
            commands.entity(face_entity).insert(Face {
                origin: face_origin,
                direction: face_direction.clone(),
                size: FaceSize {
                    w: pillar.w,
                    h: pillar.h,
                },
                tiles: face_tiles,
            });
            commands.entity(pillar_entity).add_child(face_entity);

            let face_climbers_count = face.climbers.len();
            let pylons_delta = pillar.w as f32 * TILE_SIZE / (face_climbers_count + 1) as f32;
            for (climber_idx, climber) in face.climbers.iter().enumerate() {
                // TODO Spawn pylon
                let pylon_offset = pylons_delta * (climber_idx + 1) as f32;
                let pylon_y = pillar_half_height;
                let (pylon_x, pylon_z) = match face_direction {
                    FaceDirection::West | FaceDirection::East => (
                        factor * (pillar_half_width - PYLON_HORIZONTAL_DELTA),
                        pillar_half_width - pylon_offset,
                    ),
                    FaceDirection::North | FaceDirection::South => (
                        pillar_half_width - pylon_offset,
                        factor * (pillar_half_width - PYLON_HORIZONTAL_DELTA),
                    ),
                };
                let unpowered_pylon = commands
                    .spawn((
                        PbrBundle {
                            mesh: meshes.add(
                                shape::Cylinder {
                                    radius: 0.15,
                                    height: 0.1,
                                    resolution: 16,
                                    segments: 1,
                                }
                                .into(),
                            ),
                            material: assets.pillar_mat.clone(),
                            transform: Transform::from_translation(Vec3::new(
                                pylon_x, pylon_y, pylon_z,
                            )),
                            ..default()
                        },
                        Pylon { powered: false },
                    ))
                    .id();
                unpowered_pylons
                    .get_mut(&face_direction)
                    .unwrap()
                    .push(unpowered_pylon);
                commands.entity(pillar_entity).add_child(unpowered_pylon);

                let climber_entity = spawn_climber(
                    &mut commands,
                    &assets,
                    face_entity,
                    pillar_entity,
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
        commands
            .entity(pillar_entity)
            .insert(Pillar { unpowered_pylons });
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
