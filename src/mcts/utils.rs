use crate::types::{self, GameConfig, Wall};

pub fn bresenham_line(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);

    loop {
        points.push((x, y));
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }

    points
}

pub fn has_line_of_sight(x0: i32, y0: i32, x1: i32, y1: i32, walls: &[Wall]) -> bool {
    let line = bresenham_line(x0, y0, x1, y1);

    for (x, y) in line.iter().skip(1).take(line.len().saturating_sub(2)) {
        if walls
            .iter()
            .any(|w| (x - w.x).abs() < 2 && (y - w.y).abs() < 2)
        {
            return false;
        }
    }

    true
}

pub fn is_valid_move(tx: i32, ty: i32, config: &GameConfig, walls: &[Wall]) -> bool {
    ty < config.height
        && ty > -1
        && tx < config.width
        && tx > -1
        && !walls
            .iter()
            .any(|w| (tx - w.x).abs() < 2 && (ty - w.y).abs() < 2)
}

pub fn update_bullets(state: &mut types::GameState) {
    for bullet in state.projectiles.iter_mut() {
        bullet.ttl -= 1;
    }
    state.projectiles.retain(|x| x.ttl > 0);
    for bullet in state.projectiles.iter_mut() {
        // TODO: move bullets using previous trajectory
        bullet.x += 1;
        bullet.y += 1;
    }
}

pub fn update_movement(state: &mut types::GameState, hero_id: i32, x: i32, y: i32) {
    if let Some(hero) = state.heroes.iter_mut().find(|h| h.id == hero_id) {
        hero.x = x;
        hero.y = y;
    }
}
