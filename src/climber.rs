use std::time::Duration;

use bevy::prelude::{
    default, info, Commands, Component, Entity, Name, PbrBundle, Query, Res, Transform, Vec3,
};
use bevy_tweening::{
    lens::{TransformPositionLens, TransformScaleLens},
    Animator, EaseFunction, RepeatCount, Tween,
};

use crate::{
    assets::GameAssets,
    level::{ClimberData, ClimberDirection},
    Face,
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
        tile: ClimberPosition,
        // next_tile: ClimberPosition,
        direction: ClimberDirection,
    },
    Moving {
        to: ClimberPosition,
        direction: ClimberDirection,
    },
    Falling,
    Saved,
    Dead,
}

#[derive(Component, Clone, Debug)]
pub struct Climber {
    state: ClimberState,
}

fn climber_start_falling(climber: &mut Climber) -> ClimberState {
    ClimberState::Falling
}

fn climber_start_moving(
    translation: &Vec3,
    next_translation: &Vec3,
    // tile: &ClimberPosition,
    next_tile: &ClimberPosition,
    direction: ClimberDirection,
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
        to: next_tile.clone(),
        direction,
    }
}

pub fn update_climbers(
    mut climbers: Query<(&mut Transform, &mut Climber, &mut Animator<Transform>)>,
    faces: Query<&Face>,
) {
    for (mut transform, mut climber, mut animator) in climbers.iter_mut() {
        match &climber.state {
            ClimberState::Waiting { tile, direction } => {
                let face = faces
                    .get(tile.face)
                    .expect("Climber does not appear to have a Face reference");
                // If climber doesn't have a rod beneath him anymore : falling
                if !face.has_ground_on_tile(&tile) {
                    info!("Climber falling");
                    climber.state = climber_start_falling(&mut climber);
                } else {
                    let next_tile = face.get_next_tile(tile, direction);

                    // If a rod can be reached: start moving to that rod
                    if face.has_ground_on_tile(&next_tile) {
                        info!(
                            "Climber chose a next tile : {} {} and started moving",
                            next_tile.i, next_tile.j
                        );
                        let next_pos = face.climber_get_pos_from_tile(&next_tile);
                        climber.state = climber_start_moving(
                            &transform.translation,
                            &next_pos,
                            &next_tile,
                            *direction,
                            &mut animator,
                        );
                    }
                }
            }
            ClimberState::Moving { to, direction } => {
                if animator.tweenable().progress() >= 1. {
                    info!("Climber movement done");
                    // TODO No tweening for pillars & climbers, animate according to fixed updates.
                    let face = faces
                        .get(to.face)
                        .expect("Climber does not appear to have a Face reference");

                    if to.j >= face.size.h - 1 {
                        climber.state = ClimberState::Saved;
                    } else {
                        // if reached the max width, swap direction for now
                        let direction = if (*direction == ClimberDirection::Increasing
                            && to.i >= face.size.w - 1)
                            || (*direction == ClimberDirection::Decreasing && to.i <= 0)
                        {
                            direction.get_opposite()
                        } else {
                            *direction
                        };
                        climber.state = ClimberState::Waiting {
                            tile: to.clone(),
                            direction,
                        };
                    }
                }
            }
            ClimberState::Falling => {
                // If a rod is reached : waiting
                // If the void is reached : dead
                transform.translation.y -= 0.05;
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
                tile: ClimberPosition {
                    face: face_entity,
                    i: climber_data.tile_i,
                    j: climber_data.tile_j,
                },
                // next_tile: ClimberPosition {
                //     face: face_entity,
                //     i: climber_data.next_i,
                //     j: climber_data.tile_j + 1,
                // },
                direction: climber_data.direction,
            },
        })
        .insert(Animator::new(tween))
        .insert(Name::from("Climber"))
        .id()
}
