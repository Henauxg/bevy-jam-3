use std::time::Duration;

use bevy::{
    prelude::{default, Commands, Component, Entity, Name, PbrBundle, Res, Transform, Vec3},
    ui::{FocusPolicy, Interaction},
};
use bevy_mod_picking::{highlight::Highlight, Hover, PickableMesh};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};

use crate::assets::GameAssets;

use super::TilePosition;

#[derive(Component, Clone, Debug)]
pub struct MovableRod {
    pub face: Entity,
    pub opposite_face: Entity,
    pub position: TilePosition,
}
impl MovableRod {
    pub fn swap_face(&mut self) {
        let tmp_face = self.face;
        self.face = self.opposite_face;
        self.opposite_face = tmp_face;
    }
}

#[derive(Component, Clone, Debug)]
pub struct Rod {}

pub fn spawn_movable_rod(
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

pub fn spawn_static_rod(
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
