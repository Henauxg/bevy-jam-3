use bevy::{
    prelude::{default, Assets, Color, Commands, Mesh, ResMut, Vec2, Vec3},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use rand::{distributions::Uniform, prelude::Distribution};

use warbler_grass::{
    prelude::{Grass, WarblersExplicitBundle},
    GrassConfiguration,
};

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

pub fn setup_grass(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(GrassConfiguration {
        main_color: Color::rgb(0.2, 0.5, 0.0),   //205, 38, 255, 255
        bottom_color: Color::rgb(0.1, 0.1, 0.0), //  25, 0, 12, 255
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
