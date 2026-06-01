use crate::types::{Projectile, Wall};

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

#[inline]
pub fn tile_in_wall(tx: i32, ty: i32, walls: &[Wall]) -> bool {
    walls
        .iter()
        .any(|w| (tx - w.x).abs() <= 1 && (ty - w.y).abs() <= 1)
}

pub fn has_line_of_sight(x0: i32, y0: i32, x1: i32, y1: i32, walls: &[Wall]) -> bool {
    let line = bresenham_line(x0, y0, x1, y1);
    for &(x, y) in line.iter().skip(1).take(line.len().saturating_sub(2)) {
        if tile_in_wall(x, y, walls) {
            return false;
        }
    }
    true
}

#[inline]
pub fn euclidean_dist(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    let dx = bx - ax;
    let dy = by - ay;
    (dx * dx + dy * dy).isqrt()
}

#[inline]
pub fn chebyshev_dist(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    (bx - ax).abs().max((by - ay).abs())
}

#[inline]
pub fn manhattan_dist(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    (bx - ax).abs() + (by - ay).abs()
}

pub fn projectile_danger_tiles(p: &Projectile, speed: i32) -> Vec<(i32, i32)> {
    let dx = (p.x - p.origin_x).signum();
    let dy = (p.y - p.origin_y).signum();

    if dx == 0 && dy == 0 {
        return vec![(p.x, p.y)];
    }

    let mut tiles = Vec::new();
    let mut cx = p.x;
    let mut cy = p.y;
    for _ in 0..=speed {
        tiles.push((cx, cy));
        cx += dx;
        cy += dy;
    }
    tiles
}

pub fn is_in_cover(tx: i32, ty: i32, ex: i32, ey: i32, walls: &[Wall]) -> bool {
    !has_line_of_sight(tx, ty, ex, ey, walls)
}
