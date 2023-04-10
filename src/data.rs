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

pub struct ClimberData {
    pub tile_i: u16,
    pub tile_j: u16,
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

// Can be serialized/deserialized
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
            h: 7,
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
                                j: 3,
                                kind: TileDataType::StaticRod,
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

pub fn level_1() -> LevelData {
    LevelData {
        name: "I".to_string(),
        pillars: vec![PillarData {
            w: 5,
            h: 7,
            x: 0.,
            z: 0.,
            faces: HashMap::from([
                (
                    FaceDirection::West,
                    FaceData {
                        tiles: vec![
                            TileData {
                                i: 1,
                                j: 3,
                                kind: TileDataType::StaticRod,
                            },
                            TileData {
                                i: 4,
                                j: 6,
                                kind: TileDataType::StaticRod,
                            },
                        ],
                        climbers: vec![ClimberData {
                            tile_i: 1,
                            tile_j: 3,
                        }],
                    },
                ),
                (
                    FaceDirection::East,
                    FaceData {
                        tiles: vec![
                            TileData {
                                i: 2,
                                j: 4,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 3,
                                j: 5,
                                kind: TileDataType::MovableRod,
                            },
                        ],
                        climbers: vec![],
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

pub fn level_2() -> LevelData {
    LevelData {
        name: "II".to_string(),
        pillars: vec![PillarData {
            w: 5,
            h: 7,
            x: 0.,
            z: 0.,
            faces: HashMap::from([
                (
                    FaceDirection::West,
                    FaceData {
                        tiles: vec![
                            TileData {
                                i: 1,
                                j: 3,
                                kind: TileDataType::StaticRod,
                            },
                            TileData {
                                i: 3,
                                j: 5,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 4,
                                j: 6,
                                kind: TileDataType::StaticRod,
                            },
                        ],
                        climbers: vec![ClimberData {
                            tile_i: 1,
                            tile_j: 3,
                        }],
                    },
                ),
                (
                    FaceDirection::East,
                    FaceData {
                        tiles: vec![
                            TileData {
                                i: 2,
                                j: 4,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 4,
                                j: 6,
                                kind: TileDataType::StaticRod,
                            },
                        ],
                        climbers: vec![ClimberData {
                            tile_i: 2,
                            tile_j: 4,
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

// Fall introduction
pub fn level_3() -> LevelData {
    LevelData {
        name: "III".to_string(),
        pillars: vec![PillarData {
            w: 5,
            h: 7,
            x: 0.,
            z: 0.,
            faces: HashMap::from([
                (
                    FaceDirection::West,
                    FaceData {
                        tiles: vec![
                            TileData {
                                i: 0,
                                j: 2,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 2,
                                j: 4,
                                kind: TileDataType::StaticRod,
                            },
                            TileData {
                                i: 3,
                                j: 5,
                                kind: TileDataType::StaticRod,
                            },
                            TileData {
                                i: 4,
                                j: 6,
                                kind: TileDataType::StaticRod,
                            },
                        ],
                        climbers: vec![ClimberData {
                            tile_i: 0,
                            tile_j: 2,
                        }],
                    },
                ),
                (
                    FaceDirection::East,
                    FaceData {
                        tiles: vec![
                            TileData {
                                i: 0,
                                j: 5,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 1,
                                j: 3,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 2,
                                j: 4,
                                kind: TileDataType::StaticRod,
                            },
                            TileData {
                                i: 3,
                                j: 5,
                                kind: TileDataType::StaticRod,
                            },
                            TileData {
                                i: 4,
                                j: 6,
                                kind: TileDataType::StaticRod,
                            },
                        ],
                        climbers: vec![ClimberData {
                            tile_i: 0,
                            tile_j: 5,
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

pub fn level_4() -> LevelData {
    LevelData {
        name: "IV".to_string(),
        pillars: vec![PillarData {
            w: 5,
            h: 7,
            x: 0.,
            z: 0.,
            faces: HashMap::from([
                (
                    FaceDirection::West,
                    FaceData {
                        tiles: vec![
                            TileData {
                                i: 0,
                                j: 5,
                                kind: TileDataType::StaticRod,
                            },
                            TileData {
                                i: 1,
                                j: 2,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 2,
                                j: 3,
                                kind: TileDataType::MovableRod,
                            },
                        ],
                        climbers: vec![ClimberData {
                            tile_i: 2,
                            tile_j: 3,
                        }],
                    },
                ),
                (
                    FaceDirection::East,
                    FaceData {
                        tiles: vec![
                            TileData {
                                i: 1,
                                j: 4,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 3,
                                j: 4,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 1,
                                j: 6,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 3,
                                j: 2,
                                kind: TileDataType::MovableRod,
                            },
                            TileData {
                                i: 3,
                                j: 6,
                                kind: TileDataType::StaticRod,
                            },
                            TileData {
                                i: 4,
                                j: 5,
                                kind: TileDataType::StaticRod,
                            },
                        ],
                        climbers: vec![
                            ClimberData {
                                tile_i: 1,
                                tile_j: 4,
                            },
                            ClimberData {
                                tile_i: 3,
                                tile_j: 2,
                            },
                        ],
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
