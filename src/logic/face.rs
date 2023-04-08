use bevy::prelude::{Component, Vec3};

use crate::{
    assets::{
        CLIMBER_LEVITATE_DISTANCE, CLIMBER_RADIUS, HALF_TILE_SIZE, HALF_VISIBLE_ROD_LENGTH,
        TILE_SIZE,
    },
    level::{FaceDirection, FaceSize},
    TilePosition, TileType,
};

use super::climber::ClimberPosition;

#[derive(Component, Clone, Debug)]
pub struct Face {
    pub direction: FaceDirection,
    pub size: FaceSize,
    pub origin: Vec3,
    pub tiles: Vec<Vec<TileType>>,
}

impl Face {
    pub fn is_valid(&self, i: u16, j: u16) -> bool {
        i < self.size.w && j < self.size.h
    }

    pub fn has_ground_on_tile(&self, i: u16, j: u16) -> bool {
        if !self.is_valid(i, j) {
            return false;
        }
        self.tiles[i as usize][j as usize] != TileType::Void
    }

    pub fn climber_get_pos_from_tile(&self, pos: &ClimberPosition) -> Vec3 {
        let factor = match self.direction {
            FaceDirection::West | FaceDirection::South => -1.,
            FaceDirection::North | FaceDirection::East => 1.,
        };
        let y = self.origin.y
            + pos.j as f32 * TILE_SIZE
            + TILE_SIZE
            + CLIMBER_RADIUS
            + CLIMBER_LEVITATE_DISTANCE;
        let horizontal_delta = pos.i as f32 * TILE_SIZE + HALF_TILE_SIZE;
        match self.direction {
            FaceDirection::West | FaceDirection::East => Vec3::new(
                self.origin.x + factor * HALF_VISIBLE_ROD_LENGTH,
                y,
                self.origin.z + horizontal_delta,
            ),
            FaceDirection::North | FaceDirection::South => Vec3::new(
                self.origin.x + horizontal_delta,
                y,
                self.origin.z + factor * HALF_VISIBLE_ROD_LENGTH,
            ),
        }
    }

    pub fn get_next_tile_with_ground(
        &self,
        tile: &ClimberPosition,
        // direction: &ClimberDirection,
    ) -> Option<ClimberPosition> {
        let next_tile_1 = ClimberPosition {
            face: tile.face,
            i: tile.i + 1,
            j: tile.j + 1,
        };
        if self.has_ground_on_tile(next_tile_1.i, next_tile_1.j) {
            return Some(next_tile_1);
        }
        if tile.i > 0 {
            let next_tile_2 = ClimberPosition {
                face: tile.face,
                i: tile.i - 1,
                j: tile.j + 1,
            };
            if self.has_ground_on_tile(next_tile_2.i, next_tile_2.j) {
                return Some(next_tile_2);
            }
        }

        None
    }

    // No input checks
    pub fn remove_tile_at(&mut self, pos: TilePosition) {
        self.tiles[pos.i as usize][pos.j as usize] = TileType::Void;
    }

    pub fn set_tile_at(&mut self, pos: TilePosition, tile_type: TileType) {
        self.tiles[pos.i as usize][pos.j as usize] = tile_type;
    }

    pub fn get_tile_coords_from_pos(&self, translation: Vec3) -> (u16, u16) {
        let relative = translation - self.origin;
        let j = (relative.y / TILE_SIZE).round() as u16;
        let i = match self.direction {
            FaceDirection::West | FaceDirection::East => (relative.z / TILE_SIZE).trunc(),
            FaceDirection::North | FaceDirection::South => (relative.x / TILE_SIZE).trunc(),
        } as u16;
        (i, j)
    }
}
