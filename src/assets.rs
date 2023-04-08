use bevy::prelude::{
    shape, Assets, Color, FromWorld, Handle, Mesh, Resource, StandardMaterial, World,
};

pub const TILE_SIZE: f32 = 0.5;
pub const HALF_TILE_SIZE: f32 = TILE_SIZE / 2.;

pub const DEPRECATED_PILLAR_WIDTH: f32 = 5. * TILE_SIZE;
pub const DEPRECATED_PILLAR_HEIGHT: f32 = 12. * TILE_SIZE;
pub const DEPRECATED_HALF_PILLAR_HEIGHT: f32 = DEPRECATED_PILLAR_HEIGHT / 2.;

pub const VISIBLE_ROD_LENGTH: f32 = 1.0;
pub const HALF_VISIBLE_ROD_LENGTH: f32 = VISIBLE_ROD_LENGTH / 2.;
pub const MOVABLE_ROD_LENGTH: f32 = DEPRECATED_PILLAR_WIDTH + 1.05 * VISIBLE_ROD_LENGTH; // So that it is always visible from both sides
pub const MOVABLE_ROD_MOVEMENT_AMPLITUDE: f32 = VISIBLE_ROD_LENGTH;
pub const ROD_WIDTH: f32 = 0.8 * TILE_SIZE;
pub const HALF_ROD_WIDTH: f32 = ROD_WIDTH / 2.0;

pub const PYLON_HORIZONTAL_DELTA: f32 = TILE_SIZE;

pub const CLIMBER_RADIUS: f32 = 0.15;
pub const CLIMBER_LEVITATE_DISTANCE: f32 = CLIMBER_RADIUS / 2.;

#[derive(Resource)]
pub struct GameAssets {
    pub climber_mesh: Handle<Mesh>,
    pub static_rod_mesh: Handle<Mesh>,
    pub movable_rod_mesh: Handle<Mesh>,
    pub pillar_mat: Handle<StandardMaterial>,
    pub static_rod_mat: Handle<StandardMaterial>,
    pub movable_rod_mat: Handle<StandardMaterial>,
    pub movable_rod_highlight_mat: Handle<StandardMaterial>,
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
            meshes.add(shape::Box::new(VISIBLE_ROD_LENGTH, ROD_WIDTH, ROD_WIDTH).into());
        let movable_rod_mesh =
            meshes.add(shape::Box::new(MOVABLE_ROD_LENGTH, ROD_WIDTH, ROD_WIDTH).into()); // TODO Depends on the pillars

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
            // base_color: Color::rgb(0.4, 0.3, 0.3),
            base_color: Color::WHITE,
            emissive: Color::rgb_linear(5.5, 11., 5.5),
            ..Default::default()
        });
        let movable_rod_highlight_mat = materials.add(StandardMaterial {
            perceptual_roughness: 0.9,
            metallic: 0.2,
            // base_color: Color::rgb(0.4, 0.3, 0.3),
            base_color: Color::ORANGE,
            emissive: Color::rgb_linear(6., 6., 3.),
            ..Default::default()
        });
        let climber_mat = materials.add(StandardMaterial {
            perceptual_roughness: 0.5,
            metallic: 0.2,
            // base_color: Color::LIME_GREEN,
            // emissive: Color::rgb_linear(1.0, 14., 1.32),
            // base_color: Color::ORANGE_RED,
            // emissive: Color::rgb_linear(14.0, 0.25, 0.),
            base_color: Color::BLUE,
            emissive: Color::rgb_linear(3.07, 11.22, 14.), // Light blue
            ..Default::default()
        });

        GameAssets {
            climber_mesh,
            static_rod_mesh,
            movable_rod_mesh,
            pillar_mat,
            static_rod_mat,
            movable_rod_mat,
            movable_rod_highlight_mat,
            climber_mat,
        }
    }
}
