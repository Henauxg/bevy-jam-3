use bevy::prelude::{
    shape, Assets, Color, FromWorld, Handle, Mesh, Resource, StandardMaterial, World,
};

use crate::GAME_UNIT;

pub const PILLAR_WIDTH: f32 = 5. * GAME_UNIT;
pub const HALF_PILLAR_WIDTH: f32 = PILLAR_WIDTH / 2.0;
pub const PILLAR_HEIGHT: f32 = 12. * GAME_UNIT;
pub const HALF_PILLAR_HEIGHT: f32 = PILLAR_HEIGHT / 2.;

pub const STATIC_ROD_LENGTH: f32 = PILLAR_WIDTH + 2. * GAME_UNIT;
pub const MOVABLE_ROD_LENGTH: f32 = PILLAR_WIDTH + 0.99 * GAME_UNIT; // So that it does not clip through
pub const ROD_WIDTH: f32 = GAME_UNIT / 2.0;
pub const HALF_ROD_WIDTH: f32 = ROD_WIDTH / 2.0;

pub const CLIMBER_RADIUS: f32 = 0.2;

#[derive(Resource)]
pub struct GameAssets {
    pub climber_mesh: Handle<Mesh>,
    pub static_rod_mesh: Handle<Mesh>,
    pub movable_rod_mesh: Handle<Mesh>,
    pub pillar_mat: Handle<StandardMaterial>,
    pub static_rod_mat: Handle<StandardMaterial>,
    pub movable_rod_mat: Handle<StandardMaterial>,
    pub climber_mat: Handle<StandardMaterial>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let cell = world.cell();

        let mut meshes = cell
            .get_resource_mut::<Assets<Mesh>>()
            .expect("Failed to get Assets<Mesh>");
        let climber_mesh = meshes.add(
            shape::Icosphere {
                radius: CLIMBER_RADIUS,
                subdivisions: 5,
            }
            .try_into()
            .unwrap(),
        );
        let static_rod_mesh =
            meshes.add(shape::Box::new(STATIC_ROD_LENGTH, ROD_WIDTH, ROD_WIDTH).into());
        let movable_rod_mesh =
            meshes.add(shape::Box::new(MOVABLE_ROD_LENGTH, ROD_WIDTH, ROD_WIDTH).into());

        let mut materials = cell
            .get_resource_mut::<Assets<StandardMaterial>>()
            .expect("Failed to get Assets<StandardMaterial>");

        let pillar_mat = materials.add(StandardMaterial {
            perceptual_roughness: 0.9,
            metallic: 0.2,
            base_color: Color::rgb(0.8, 0.7, 0.6),
            ..Default::default()
        });
        let static_rod_mat = pillar_mat.clone();
        let movable_rod_mat = materials.add(StandardMaterial {
            perceptual_roughness: 0.9,
            metallic: 0.2,
            base_color: Color::rgb(0.4, 0.3, 0.3),
            ..Default::default()
        });
        let climber_mat = materials.add(StandardMaterial {
            perceptual_roughness: 0.9,
            metallic: 0.2,
            base_color: Color::rgb(0.4, 0.7, 0.4),
            ..Default::default()
        });

        GameAssets {
            climber_mesh,
            static_rod_mesh,
            movable_rod_mesh,
            pillar_mat,
            static_rod_mat,
            movable_rod_mat,
            climber_mat,
        }
    }
}
