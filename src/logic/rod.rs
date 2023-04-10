use std::time::Duration;

use bevy::{
    prelude::{
        default, Commands, Component, Entity, EventReader, Name, PbrBundle, Query, Res, Transform,
        Vec3,
    },
    ui::{FocusPolicy, Interaction},
};
use bevy_mod_picking::{highlight::Highlight, Hover, PickableMesh, PickingEvent, SelectionEvent};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};

use crate::assets::GameAssets;

use super::{face::Face, TilePosition, TileType};

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

pub fn handle_movable_rod_picking_events(
    mut events: EventReader<PickingEvent>,
    mut rods_animators: Query<(&Transform, &mut Animator<Transform>, &mut MovableRod)>,
    // mut win_pylon: Query<(), With<WinPylon>>,
    mut faces: Query<&mut Face>,
    // level_completion: Res<LevelCompletion>,
    // mut level_events: EventWriter<LevelEvent>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(SelectionEvent::JustSelected(_)) => {}
            PickingEvent::Selection(SelectionEvent::JustDeselected(_)) => {}
            PickingEvent::Hover(_) => {}
            PickingEvent::Clicked(entity) => {
                if let Ok((rod_transform, mut rod_animator, mut rod)) =
                    rods_animators.get_mut(*entity)
                {
                    if rod_animator.tweenable().progress() >= 1.0 {
                        // Immediately set void for this face
                        let mut face = faces
                            .get_mut(rod.face)
                            .expect("Rod does not appear to have a Face reference");
                        face.remove_tile_at(rod.position);

                        let mut opposite_face = faces
                            .get_mut(rod.opposite_face)
                            .expect("Rod does not appear to have a Face reference");
                        // TODO set MovingRod on the other face after a delay (animation duration / 2)
                        opposite_face.set_tile_at(rod.position, TileType::MovableRod(false));

                        rod.swap_face();

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
