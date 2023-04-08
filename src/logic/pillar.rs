use std::collections::HashMap;

use bevy::prelude::{
    default, shape, Assets, Commands, Component, Entity, Mesh, Name, PbrBundle, Res, ResMut,
    Transform, Vec3,
};

use crate::{
    assets::{GameAssets, TILE_SIZE},
    level::{FaceDirection, PillarData},
};

#[derive(Clone, Debug)]
pub struct PillarFace {
    // pub size: FaceSize,
    // pub tiles: Vec<Vec<TileType>>,
}

#[derive(Component, Clone, Debug)]
pub struct Pillar {
    // pub faces: Vec<PillarFace>,
    pub unpowered_pylons: HashMap<FaceDirection, Vec<Entity>>,
}

impl Pillar {
    pub fn get_pylon_from_face(&mut self, dir: &FaceDirection) -> Option<Entity> {
        let pylons = self.unpowered_pylons.get_mut(&dir).unwrap();
        if pylons.len() > 0 {
            return Some(pylons.pop().unwrap());
        }
        None
    }

    pub fn pop_first_available_pylon(&mut self) -> Option<Entity> {
        for dir in [
            FaceDirection::East,
            FaceDirection::West,
            FaceDirection::South,
            FaceDirection::North,
        ] {
            let pylons = self.unpowered_pylons.get_mut(&dir).unwrap();
            if pylons.len() > 0 {
                return Some(pylons.pop().unwrap());
            }
        }
        None
    }
}

pub fn spawn_pillar(
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
            Name::from("Pillar"),
        ))
        .id()
}
