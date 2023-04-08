use std::collections::HashMap;

use bevy::prelude::{AmbientLight, Color};

#[derive(Clone, Copy, Debug)]
pub enum TileDataType {
    StaticRod,
    MovableRod,
}

pub struct TileData {
    pub i: u16,
    pub j: u16,
    pub kind: TileDataType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaceSize {
    pub w: u16,
    pub h: u16,
}

// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// pub enum ClimberDirection {
//     Increasing,
//     Decreasing,
// }
// impl ClimberDirection {
//     pub fn get_opposite(&self) -> ClimberDirection {
//         match self {
//             ClimberDirection::Increasing => ClimberDirection::Decreasing,
//             ClimberDirection::Decreasing => ClimberDirection::Increasing,
//         }
//     }
// }

pub struct ClimberData {
    pub tile_i: u16,
    pub tile_j: u16,
    // pub direction: ClimberDirection,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FaceDirection {
    West,
    North,
    East,
    South,
}

impl FaceDirection {
    pub fn get_opposite(&self) -> FaceDirection {
        match self {
            FaceDirection::West => FaceDirection::East,
            FaceDirection::North => FaceDirection::South,
            FaceDirection::East => FaceDirection::West,
            FaceDirection::South => FaceDirection::North,
        }
    }
}

pub struct FaceData {
    // pub h_offset: f32,
    // pub w_offset: f32,
    // pub size: FaceSize,
    pub tiles: Vec<TileData>,
    pub climbers: Vec<ClimberData>,
}

pub struct PillarData {
    pub x: f32,
    pub z: f32,
    pub w: u16,
    pub h: u16,
    pub faces: HashMap<FaceDirection, FaceData>,
}

pub struct LevelData {
    pub name: String,
    pub pillars: Vec<PillarData>,
    pub background_color: Color,
    pub dir_light_color: Color,
    pub ambient_light: AmbientLight,
}

pub fn test_level_data() -> LevelData {
    LevelData {
        name: "Test level".to_string(),
        pillars: vec![PillarData {
            w: 5,
            h: 10,
            x: 0.,
            z: 0.,
            faces: HashMap::from([
                (
                    FaceDirection::West,
                    FaceData {
                        tiles: vec![
                            TileData {
                                i: 0,
                                j: 0,
                                kind: TileDataType::StaticRod,
                            },
                            TileData {
                                i: 1,
                                j: 1,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 3,
                                j: 3,
                                kind: TileDataType::StaticRod,
                            },
                            TileData {
                                i: 4,
                                j: 4,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 4,
                                j: 6,
                                kind: TileDataType::MovableRod,
                            },
                        ],
                        climbers: vec![ClimberData {
                            tile_i: 0,
                            tile_j: 0,
                            // direction: ClimberDirection::Increasing,
                        }],
                    },
                ),
                (
                    FaceDirection::East,
                    FaceData {
                        tiles: vec![
                            TileData {
                                i: 0,
                                j: 0,
                                kind: TileDataType::StaticRod,
                            },
                            TileData {
                                i: 2,
                                j: 2,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 3,
                                j: 5,
                                kind: TileDataType::MovableRod,
                            },
                        ],
                        climbers: vec![ClimberData {
                            tile_i: 0,
                            tile_j: 0,
                            // direction: ClimberDirection::Increasing,
                        }],
                    },
                ),
            ]),
        }],
        background_color: Color::TURQUOISE,
        dir_light_color: Color::ORANGE,
        ambient_light: AmbientLight {
            color: Color::ORANGE_RED,
            brightness: 0.2,
        },
    }
}
