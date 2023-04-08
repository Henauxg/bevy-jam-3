use std::time::Duration;

use bevy::prelude::{
    default, info, BuildChildren, Commands, Component, Entity, Handle, Name, PbrBundle, Query, Res,
    StandardMaterial, Transform, Vec3, Without,
};
use bevy_tweening::{
    lens::{TransformPositionLens, TransformScaleLens},
    Animator, EaseFunction, RepeatCount, Tween,
};

use crate::{
    assets::{GameAssets, CLIMBER_LEVITATE_DISTANCE, CLIMBER_RADIUS, TILE_SIZE},
    data::ClimberData,
    Face, Pillar, Pylon,
};

#[derive(Clone, Debug)]
pub struct ClimberPosition {
    pub face: Entity,
    pub i: u16,
    pub j: u16,
}

#[derive(Clone, Debug)]
enum ClimberState {
    Waiting {
        on_tile: ClimberPosition,
        // next_tile: ClimberPosition,
        // direction: ClimberDirection,
    },
    Moving {
        to_tile: ClimberPosition,
        // direction: ClimberDirection,
    },
    Falling {
        on_face: Entity,
    },
    Saved,
    Dead,
}

#[derive(Component, Clone, Debug)]
pub struct Climber {
    state: ClimberState,
    current_pillar: Entity,
}

fn climber_start_moving(
    translation: &Vec3,
    next_translation: &Vec3,
    // tile: &ClimberPosition,
    next_tile: &ClimberPosition,
    // direction: ClimberDirection,
    animator: &mut Animator<Transform>,
) -> ClimberState {
    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(400),
        TransformScaleLens {
            start: Vec3::ONE,
            end: Vec3::ZERO,
        },
    )
    .then(Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(1),
        TransformPositionLens {
            start: translation.clone(),
            end: next_translation.clone(),
        },
    ))
    .then(
        Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(400),
            TransformScaleLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        ), // .with_completed_event(),
    );
    animator.set_tweenable(tween);

    ClimberState::Moving {
        to_tile: next_tile.clone(),
        // direction,
    }
}

pub fn update_climbers(
    mut commands: Commands,
    mut climbers: Query<(
        &mut Transform,
        &mut Climber,
        &mut Animator<Transform>,
        Entity,
    )>,
    faces: Query<&Face>,
    mut pillars: Query<&mut Pillar>,
    mut pylons: Query<
        (&mut Pylon, &mut Transform, &mut Handle<StandardMaterial>),
        Without<Climber>,
    >,
    assets: Res<GameAssets>,
) {
    for (mut transform, mut climber, mut animator, climber_entity) in climbers.iter_mut() {
        match &climber.state {
            ClimberState::Waiting { on_tile: tile } => {
                let face = faces
                    .get(tile.face)
                    .expect("Climber does not appear to have a Face reference");
                // If climber doesn't have a rod beneath him anymore : falling
                if !face.has_ground_on_tile(tile.i, tile.j) {
                    info!("Climber started falling from {} {}", tile.i, tile.j);
                    climber.state = ClimberState::Falling { on_face: tile.face };
                } else {
                    if let Some(next_tile) = face.get_next_tile_with_ground(tile) {
                        let next_pos = face.climber_get_pos_from_tile(&next_tile);
                        info!(
                            "Climber chose a next tile {} {} from {} {} and started moving",
                            next_tile.i, next_tile.j, tile.i, tile.j
                        );
                        climber.state = climber_start_moving(
                            &transform.translation,
                            &next_pos,
                            &next_tile,
                            // *direction,
                            &mut animator,
                        );
                    }
                }
            }
            ClimberState::Moving { to_tile: to } => {
                // "Clean code"
                if animator.tweenable().progress() >= 1. {
                    info!("Climber movement done");
                    // TODO No tweening for pillars & climbers, animate according to fixed updates.
                    let face = faces
                        .get(to.face)
                        .expect("Climber does not appear to have a Face reference");

                    if to.j >= face.size.h - 1 {
                        let mut pillar = pillars.get_mut(climber.current_pillar).unwrap();
                        let pylon_entity = if let Some(same_face_pylon) =
                            pillar.get_pylon_from_face(&face.direction)
                        {
                            same_face_pylon
                        } else {
                            pillar.pop_first_available_pylon().unwrap()
                        };
                        // TODO Pylon as full cylinder
                        let (mut pylon, pylon_transform, mut mat_handle) =
                            pylons.get_mut(pylon_entity).unwrap();
                        pylon.powered = true;
                        *mat_handle = assets.climber_mat.clone();

                        let pos = pylon_transform.translation;
                        let tween = Tween::new(
                            EaseFunction::QuadraticInOut,
                            Duration::from_millis(3000),
                            TransformPositionLens {
                                start: pos,
                                end: Vec3::new(pos.x, pos.y + TILE_SIZE, pos.z),
                            },
                        );
                        commands.entity(pylon_entity).insert(Animator::new(tween));
                        transform.translation =
                            Vec3::new(0., CLIMBER_RADIUS + CLIMBER_LEVITATE_DISTANCE, 0.);
                        commands.entity(pylon_entity).add_child(climber_entity);
                        climber.state = ClimberState::Saved;
                    } else {
                        climber.state = ClimberState::Waiting {
                            on_tile: to.clone(),
                        };
                    }
                }
            }
            ClimberState::Falling {
                on_face: face_entity,
            } => {
                // If a rod is reached : waiting
                let face = faces
                    .get(*face_entity)
                    .expect("Climber does not appear to have a Face reference");

                let (i, j) = face.get_tile_coords_from_pos(transform.translation);
                if face.has_ground_on_tile(i, j) {
                    info!("Climber landed on tile {} {}", i, j);
                    let landed_on = ClimberPosition {
                        face: *face_entity,
                        i,
                        j,
                    };
                    transform.translation = face.climber_get_pos_from_tile(&landed_on);
                    climber.state = ClimberState::Waiting { on_tile: landed_on };
                } else {
                    transform.translation.y -= 0.05;
                }

                // If the ground is reached : dead
                if transform.translation.y <= 0.0 {
                    climber.state = ClimberState::Dead;
                }
            }
            ClimberState::Saved => {
                // TODO Win
            }
            ClimberState::Dead => {
                // TODO Lost
            }
        }
    }
}

pub fn spawn_climber(
    commands: &mut Commands,
    assets: &Res<GameAssets>,
    face_entity: Entity,
    pillar_entity: Entity,
    climber_data: &ClimberData,
    x: f32,
    y: f32,
    z: f32,
) -> Entity {
    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(1200),
        TransformScaleLens {
            start: Vec3::new(0.9, 0.9, 0.9),
            end: Vec3::new(1.1, 1.1, 1.1),
        },
    )
    .with_repeat_count(RepeatCount::Infinite)
    .with_repeat_strategy(bevy_tweening::RepeatStrategy::MirroredRepeat);

    commands
        .spawn((PbrBundle {
            mesh: assets.climber_mesh.clone(),
            material: assets.climber_mat.clone(),
            transform: Transform::from_xyz(x, y, z),
            ..default()
        },))
        .insert(Climber {
            state: ClimberState::Waiting {
                on_tile: ClimberPosition {
                    face: face_entity,
                    i: climber_data.tile_i,
                    j: climber_data.tile_j,
                },
            },
            current_pillar: pillar_entity,
        })
        .insert(Animator::new(tween))
        .insert(Name::from("Climber"))
        .id()
}
