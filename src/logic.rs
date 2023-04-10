use bevy::prelude::{Component, EventReader, EventWriter, Query, Res, With};
use bevy_mod_picking::{PickingEvent, SelectionEvent};

use self::level::{LevelCompletion, LevelEvent};

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
    StaticRod(bool),
    MovableRod(bool),
}

// #[derive(Clone, Debug)]
// pub struct TileData {
//     pub kind: TileType,
// }

#[derive(Component, Clone, Debug)]
pub struct Pylon {
    pub powered: bool,
}

#[derive(Component, Clone, Debug)]
pub struct WinPylon;

pub fn handle_win_pylon_pick_events(
    mut events: EventReader<PickingEvent>,
    mut win_pylon: Query<(), With<WinPylon>>,
    level_completion: Res<LevelCompletion>,
    mut level_events: EventWriter<LevelEvent>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(SelectionEvent::JustSelected(_)) => {}
            PickingEvent::Selection(SelectionEvent::JustDeselected(_)) => {}
            PickingEvent::Hover(_) => {}
            PickingEvent::Clicked(entity) => {
                if let Ok(_) = win_pylon.get_mut(*entity) {
                    if level_completion.is_won() {
                        level_events.send(LevelEvent::LoadNext);
                    }
                }
            }
        }
    }
}
