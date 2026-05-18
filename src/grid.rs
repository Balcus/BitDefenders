use crate::types::{self, Wall};

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub is_wall: bool,
    // TODO: this mignt not be needed, consider exploration later
    pub should_consider: bool,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            is_wall: false,
            should_consider: true,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Grid {
    pub tiles: Vec<Vec<Tile>>,
}

impl Grid {
    pub fn from(width: i32, height: i32, walls: &[Wall]) -> Self {
        let mut grid = Self::default();

        grid.tiles = vec![vec![Tile::default(); height as usize]; width as usize];
        for wall in walls {
            grid.tiles[wall.x as usize][wall.y as usize].should_consider = false;
            grid.tiles[wall.x as usize][wall.y as usize].is_wall = true;
        }
        grid
    }

    pub fn update(&mut self, state: types::GameState) {
        for wall in &state.walls {
            self.tiles[wall.x as usize][wall.y as usize].should_consider = false;
            self.tiles[wall.x as usize][wall.y as usize].is_wall = true;
        }
    }
}
