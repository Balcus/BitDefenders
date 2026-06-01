use super::{
    constants::*,
    geometry::{
        euclidean_dist, has_line_of_sight, manhattan_dist, projectile_danger_tiles, tile_in_wall,
    },
};
use crate::{
    ai::grid::Grid,
    types::{EnemySide, GameConfig, Hero, Projectile, Wall},
};

pub struct EvalContext<'a> {
    pub hero: &'a Hero,
    pub enemies: &'a [&'a Hero],
    pub allies: &'a [&'a Hero],
    pub walls: &'a [Wall],
    pub projectiles: &'a [Projectile],
    pub config: &'a GameConfig,
    pub enemy_side: Option<EnemySide>,
    pub grid: &'a Grid,
    pub proj_speed: i32,
    pub max_hp: i32,
}

impl<'a> EvalContext<'a> {
    pub fn new(
        hero: &'a Hero,
        enemies: &'a [&'a Hero],
        allies: &'a [&'a Hero],
        walls: &'a [Wall],
        projectiles: &'a [Projectile],
        config: &'a GameConfig,
        enemy_side: Option<EnemySide>,
        grid: &'a Grid,
    ) -> Self {
        let sniper = config.hero_types.get("sniper");
        let proj_speed = sniper.map(|s| s.projectile_speed).unwrap_or(5);
        let max_hp = sniper.map(|s| s.max_hp).unwrap_or(100);
        Self {
            hero,
            enemies,
            allies,
            walls,
            projectiles,
            config,
            enemy_side,
            grid,
            proj_speed,
            max_hp,
        }
    }

    pub fn is_low_hp(&self) -> bool {
        (self.hero.hp as f32) < (self.max_hp as f32) * LOW_HP_FRACTION
    }
}

pub fn is_valid_move(tx: i32, ty: i32, ctx: &EvalContext) -> bool {
    tx >= 0
        && tx < ctx.config.width
        && ty >= 0
        && ty < ctx.config.height
        && !tile_in_wall(tx, ty, ctx.walls)
}

pub fn eval_pos(tx: i32, ty: i32, ctx: &EvalContext) -> i32 {
    let mut score = 0;

    if ctx.grid.tiles[tx as usize][ty as usize].should_consider {
        score += EXPLORATION_BONUS;
    }

    score += advance_bonus(ty, ctx);

    if ctx.hero.x == tx && ctx.hero.y == ty {
        score -= DONT_MOVE_PENALTY;
    }

    for ally in ctx.allies {
        if manhattan_dist(tx, ty, ally.x, ally.y) <= ALLY_CLUSTER_DIST {
            score += CLOSE_TOGETHER_BONUS;
        }
    }

    if tile_in_wall(tx, ty, ctx.walls) {
        score -= MOVE_INTO_WALL_PENALTY;
    }

    score += projectile_danger_score(tx, ty, ctx);
    score += enemy_interaction_score(tx, ty, ctx);

    score
}

fn advance_bonus(ty: i32, ctx: &EvalContext) -> i32 {
    let h = ctx.config.height;
    let enemy_y = match ctx.enemy_side {
        Some(EnemySide::Top) => 0,
        Some(EnemySide::Bottom) => h - 1,
        None => h / 2,
    };
    let dist = (ty - enemy_y).abs();
    (h - dist) * 20 / h.max(1)
}

fn projectile_danger_score(tx: i32, ty: i32, ctx: &EvalContext) -> i32 {
    let mut score = 0;
    for p in ctx.projectiles {
        if (tx - p.x).abs() <= 1 && (ty - p.y).abs() <= 1 {
            score -= HIT_BY_PROJECTILE_PENALTY;
        }
        for &(px, py) in &projectile_danger_tiles(p, ctx.proj_speed) {
            if (tx - px).abs() <= 1 && (ty - py).abs() <= 1 {
                score -= FUTURE_PROJECTILE_PENALTY;
                break;
            }
        }
    }
    score
}

fn enemy_interaction_score(tx: i32, ty: i32, ctx: &EvalContext) -> i32 {
    let mut score = 0;
    let low_hp = ctx.is_low_hp();

    for enemy in ctx.enemies {
        let has_los = has_line_of_sight(tx, ty, enemy.x, enemy.y, ctx.walls);

        if has_los {
            score -= ENEMY_SEES_U_PENALTY;
            if low_hp {
                score -= LOW_HP_EXPOSURE_PENALTY;
            }
            score += HAVE_LOS_ON_ENEMY_BONUS;
        } else {
            score += IN_COVER_BONUS;
        }

        let dist = euclidean_dist(tx, ty, enemy.x, enemy.y);
        if low_hp {
            score += dist / 4;
        } else {
            score -= dist * APPROACH_ENEMY_WEIGHT;
        }
    }

    score
}

pub fn best_reachable_score(tx: i32, ty: i32, ctx: &EvalContext, depth: u32) -> i32 {
    if depth == 0 {
        return 0;
    }

    const MOVES: [(i32, i32); 9] = [
        (0, 0),
        (0, 1),
        (0, -1),
        (1, 0),
        (-1, 0),
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
    ];

    let mut best = i32::MIN;
    for (dx, dy) in MOVES {
        let nx = tx + dx * 3;
        let ny = ty + dy * 3;
        if is_valid_move(nx, ny, ctx) {
            let score = eval_pos(nx, ny, ctx) + best_reachable_score(nx, ny, ctx, depth - 1);
            if score > best {
                best = score;
            }
        }
    }

    if best == i32::MIN { -1000 } else { best }
}

pub fn shoot_score(_hero: &Hero, enemy: &Hero, config: &GameConfig) -> i32 {
    let mut score = SHOOT_SCORE;
    let max_hp = config
        .hero_types
        .get("sniper")
        .map(|s| s.max_hp)
        .unwrap_or(100);
    if (enemy.hp as f32 / max_hp as f32) < 0.5 {
        score += SHOOT_LOW_HP_BONUS;
    }
    score
}
