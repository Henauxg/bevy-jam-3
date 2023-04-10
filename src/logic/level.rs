use std::{collections::HashMap, time::Duration};

use bevy::{
    pbr::CascadeShadowConfigBuilder,
    prelude::{
        default, shape, Assets, BuildChildren, Bundle, Color, Commands, Component,
        DespawnRecursiveExt, DirectionalLight, DirectionalLightBundle, Entity, EulerRot,
        EventReader, Mesh, Name, NextState, PbrBundle, Quat, Query, Res, ResMut, Resource,
        SpatialBundle, Transform, Vec3, With,
    },
    ui::{FocusPolicy, Interaction},
};
use bevy_mod_picking::{highlight::Highlight, Hover, PickableMesh};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};

use crate::{
    assets::{
        GameAssets, CLIMBER_LEVITATE_DISTANCE, CLIMBER_RADIUS, HALF_ROD_WIDTH, HALF_TILE_SIZE,
        HALF_VISIBLE_ROD_LENGTH, MOVABLE_ROD_MOVEMENT_AMPLITUDE, PYLON_HEIGHT,
        PYLON_HORIZONTAL_DELTA, PYLON_RADIUS, TILE_SIZE, WIN_PYLON_ANIMATION_DURATION,
        WIN_PYLON_HEIGHT,
    },
    data::{FaceDirection, FaceSize, LevelData, TileDataType},
    GameState,
};

use super::{
    climber::{spawn_climber, ClimberEvent},
    face::Face,
    pillar::{spawn_pillar, Pillar},
    rod::{spawn_movable_rod, spawn_static_rod},
    Pylon, TilePosition, TileType, WinPylon,
};

#[derive(Component, Default)]
pub struct LevelName(pub String);

#[derive(Bundle, Default)]
pub struct Levelbundle {
    name: Name,
    spatial: SpatialBundle,
    level_name: LevelName,
}

impl Levelbundle {
    pub fn new(name: &str) -> Self {
        Self {
            name: Name::from(name),
            level_name: LevelName(name.to_string()),
            ..default()
        }
    }
}

pub enum LevelEvent {
    Reload,
    LoadNext,
}

#[derive(Resource)]
pub struct GameLevels {
    pub current_level_entity: Option<Entity>,

    current_level_idx: usize,
    level_builders: Vec<fn() -> LevelData>,
}

#[derive(Resource)]
pub struct LevelCompletion {
    pub pylons_count: u8,
    pub powered_pylons_count: u8,
}
impl LevelCompletion {
    pub fn is_won(&self) -> bool {
        self.powered_pylons_count >= self.pylons_count
    }
}

impl GameLevels {
    pub fn new(level_builders: Vec<fn() -> LevelData>) -> Self {
        Self {
            current_level_idx: 0,
            current_level_entity: None,
            level_builders,
        }
    }
    pub fn advance_level(&mut self) {
        self.current_level_idx = (self.current_level_idx + 1) % self.level_builders.len();
    }

    pub fn get_current_level_data(&self) -> LevelData {
        self.level_builders[self.current_level_idx]()
    }
}

pub fn level_event_handler(
    mut level_events: EventReader<LevelEvent>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    mut game_levels: ResMut<GameLevels>,
    assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in level_events.iter() {
        match event {
            LevelEvent::Reload | LevelEvent::LoadNext => {
                if let Some(level_entity) = game_levels.current_level_entity.take() {
                    commands.entity(level_entity).despawn_recursive();
                }
                match event {
                    LevelEvent::LoadNext => game_levels.advance_level(),
                    _ => (),
                }
                game_levels.current_level_entity = Some(spawn_level(
                    &game_levels.get_current_level_data(),
                    commands,
                    meshes,
                    assets,
                    // materials,
                ));
                next_state.set(GameState::Playing);
            }
        }
        break;
    }
    level_events.clear();
}

pub fn climber_event_handler(
    mut commands: Commands,
    mut climber_events: EventReader<ClimberEvent>,
    mut level_completion: ResMut<LevelCompletion>,
    mut win_pylon: Query<(&mut Transform, Entity), With<WinPylon>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in climber_events.iter() {
        match event {
            ClimberEvent::ReachedTop => {
                level_completion.powered_pylons_count += 1;
                if level_completion.is_won() {
                    let (win_pylon_transform, win_pylon_entity) = win_pylon.single_mut();
                    let pos = win_pylon_transform.translation;
                    let tween = Tween::new(
                        EaseFunction::QuadraticInOut,
                        Duration::from_millis(WIN_PYLON_ANIMATION_DURATION),
                        TransformPositionLens {
                            start: pos,
                            end: Vec3::new(pos.x, pos.y + TILE_SIZE, pos.z),
                        },
                    );
                    commands
                        .entity(win_pylon_entity)
                        .insert(Animator::new(tween));
                    next_state.set(GameState::Won);
                }
            }
        }
    }
}

pub fn spawn_level(
    level_data: &LevelData,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<GameAssets>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let level_entity = commands.spawn(Levelbundle::new(&level_data.name)).id();

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

    for pillar in level_data.pillars.iter() {
        let pillar_entity = spawn_pillar(&mut commands, &mut meshes, &assets, &pillar);
        commands.entity(level_entity).add_child(pillar_entity);

        let pillar_half_width = pillar.w as f32 * TILE_SIZE / 2.;
        let pillar_half_height = pillar.h as f32 * TILE_SIZE / 2.;

        let win_pylon = commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(
                        shape::Cylinder {
                            radius: pillar_half_width / 3.,
                            height: WIN_PYLON_HEIGHT,
                            resolution: 24,
                            segments: 1,
                        }
                        .into(),
                    ),
                    material: assets.climber_mat.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        0.,
                        pillar_half_height - 1.05 * WIN_PYLON_HEIGHT / 2.,
                        0.,
                    )),
                    ..default()
                },
                WinPylon,
                Highlight::default(),
                Hover::default(),
                FocusPolicy::Block,
                Interaction::default(),
                PickableMesh::default(),
            ))
            .id();
        commands.entity(pillar_entity).add_child(win_pylon);

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
        for face_entity in face_entities.values() {
            commands.entity(pillar_entity).add_child(*face_entity);
        }

        let mut unpowered_pylons: HashMap<FaceDirection, Vec<Entity>> = HashMap::from([
            (FaceDirection::East, vec![]),
            (FaceDirection::West, vec![]),
            (FaceDirection::North, vec![]),
            (FaceDirection::South, vec![]),
        ]);
        let mut unpowered_pylons_count: u8 = 0;
        for (face_direction, face) in pillar.faces.iter() {
            let &face_entity = face_entities.get(&face_direction).unwrap();
            let opposite_face_entity = face_entities.get(&face_direction.get_opposite()).unwrap();
            let factor = match face_direction {
                FaceDirection::West => -1.,
                FaceDirection::North => 1., // TODO North south
                FaceDirection::East => 1.,
                FaceDirection::South => -1., // TODO North south
            };

            // move on x axis for west/east
            // move on z axis for north/south

            let col = vec![TileType::Void; pillar.h as usize];
            let mut face_tiles = vec![col; pillar.w as usize];
            for tile in face.tiles.iter() {
                let tile_entity = match tile.kind {
                    // Relative to pillar position.
                    TileDataType::StaticRod => {
                        face_tiles[tile.i as usize][tile.j as usize] = TileType::StaticRod(false);
                        spawn_static_rod(
                            &mut commands,
                            &assets,
                            factor * (HALF_VISIBLE_ROD_LENGTH + pillar_half_width), // TODO North south
                            tile.j as f32 * TILE_SIZE + HALF_TILE_SIZE - pillar_half_height, // TODO + HALF_PILLAR_WIDTH ? Where is the origin of the 3d mesh ?
                            tile.i as f32 * TILE_SIZE + HALF_TILE_SIZE - pillar_half_width,
                        )
                    }
                    TileDataType::MovableRod => {
                        face_tiles[tile.i as usize][tile.j as usize] = TileType::MovableRod(false);
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

            let face_climbers_count = face.climbers.len();
            let pylons_delta = pillar.w as f32 * TILE_SIZE / (face_climbers_count + 1) as f32;
            for (climber_idx, climber) in face.climbers.iter().enumerate() {
                // TODO Spawn pylon
                let pylon_offset = pylons_delta * (climber_idx + 1) as f32;
                let pylon_y = pillar_half_height - 0.8 * PYLON_HEIGHT / 2.;
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
                                    radius: PYLON_RADIUS,
                                    height: PYLON_HEIGHT,
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
                unpowered_pylons_count += 1;
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
                    (climber.tile_i as f32) * TILE_SIZE - pillar_half_width + HALF_TILE_SIZE,
                );
                commands.entity(level_entity).add_child(climber_entity);

                let tile_data = face_tiles[climber.tile_i as usize][climber.tile_j as usize];
                face_tiles[climber.tile_i as usize][climber.tile_j as usize] = match tile_data {
                    TileType::Void => TileType::Void,
                    TileType::StaticRod(_) => TileType::StaticRod(true),
                    TileType::MovableRod(_) => TileType::MovableRod(true),
                };
            }

            commands.entity(face_entity).insert(Face {
                origin: face_origin,
                direction: face_direction.clone(),
                size: FaceSize {
                    w: pillar.w,
                    h: pillar.h,
                },
                tiles: face_tiles,
            });
        }
        commands
            .entity(pillar_entity)
            .insert(Pillar { unpowered_pylons });

        commands.insert_resource(LevelCompletion {
            pylons_count: unpowered_pylons_count,
            powered_pylons_count: 0,
        });
    }

    level_entity
}
