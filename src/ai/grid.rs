use crate::types::Wall;

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub is_wall: bool,
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

        for (xi, col) in grid.tiles.iter_mut().enumerate() {
            for (yi, tile) in col.iter_mut().enumerate() {
                tile.x = xi as i32;
                tile.y = yi as i32;
            }
        }

        for wall in walls {
            Self::mark_wall_footprint(&mut grid.tiles, wall.x, wall.y);
        }

        grid
    }

    pub fn update_walls(&mut self, walls: &[Wall]) {
        for wall in walls {
            Self::mark_wall_footprint(&mut self.tiles, wall.x, wall.y);
        }
    }

    fn mark_wall_footprint(tiles: &mut Vec<Vec<Tile>>, cx: i32, cy: i32) {
        let w = tiles.len() as i32;
        let h = tiles.first().map(|c| c.len() as i32).unwrap_or(0);
        for dx in -1..=1_i32 {
            for dy in -1..=1_i32 {
                let tx = cx + dx;
                let ty = cy + dy;
                if tx >= 0 && tx < w && ty >= 0 && ty < h {
                    tiles[tx as usize][ty as usize].should_consider = false;
                    tiles[tx as usize][ty as usize].is_wall = true;
                }
            }
        }
    }
}
