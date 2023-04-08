use std::time::Duration;

use bevy::prelude::{info, Component, EventReader, Query, Transform, Vec3};
use bevy_mod_picking::{PickingEvent, SelectionEvent};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};

use self::{face::Face, rod::MovableRod};

pub mod climber;
pub mod face;
pub mod level;
pub mod pillar;
pub mod rod;

#[derive(Clone, Copy, Debug)]
pub struct TilePosition {
    pub i: u16,
    pub j: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileType {
    Void,
    StaticRod,
    MovableRod,
}

// #[derive(Clone, Debug)]
// pub struct TileData {
//     pub kind: TileType,
// }

#[derive(Component, Clone, Debug)]
pub struct Pylon {
    pub powered: bool,
}

pub fn handle_picking_events(
    mut events: EventReader<PickingEvent>,
    mut rods_animators: Query<(&Transform, &mut Animator<Transform>, &mut MovableRod)>,
    mut faces: Query<&mut Face>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(SelectionEvent::JustSelected(entity)) => {
                info!("SelectionEvent JustSelected {:?}", entity);
            }
            PickingEvent::Selection(SelectionEvent::JustDeselected(entity)) => {
                info!("SelectionEvent JustDeselected {:?}", entity);
            }
            PickingEvent::Hover(_) => {}
            PickingEvent::Clicked(entity) => {
                if let Ok((rod_transform, mut rod_animator, mut rod)) =
                    rods_animators.get_mut(*entity)
                {
                    // TODO Add a criteria here

                    // Immediately set void for this face
                    let mut face = faces
                        .get_mut(rod.face)
                        .expect("Rod does not appear to have a Face reference");
                    face.remove_tile_at(rod.position);

                    let mut opposite_face = faces
                        .get_mut(rod.opposite_face)
                        .expect("Rod does not appear to have a Face reference");
                    // TODO set MovingRod on the other face after a delay (animation duration / 2)
                    opposite_face.set_tile_at(rod.position, TileType::MovableRod);

                    rod.swap_face();

                    if rod_animator.tweenable().progress() >= 1.0 {
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
