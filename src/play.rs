use crate::types::{
    Action, EnemySide, GameConfig, GameState, Hero, MoveArgs, Projectile, ShootArgs, Wall,
};

const POSSIBLE_MOVES: [(i32, i32); 9] = [
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

const MIN_SCORE: i32 = i32::MIN;
const MOVE_INTO_WALL_PENALTY: i32 = 25;
const HIT_BY_PROJECTILE_PENALTY: i32 = 500;
const DONT_MOVE_PENALTY: i32 = 25;
const SHOOT_SCORE: i32 = 450;
const CLOSE_TOGHETER_BONUS: i32 = 0;

pub struct Tile {
    _x: i32,
    _y: i32,
}

pub struct EvalContext<'a> {
    hero: &'a Hero,
    _enemeies: &'a [&'a Hero],
    allies: &'a [&'a Hero],
    walls: &'a [Wall],
    projectiles: &'a [Projectile],
    config: &'a GameConfig,
    enemy_side: Option<EnemySide>,
}

pub fn decide_actions(
    player_id: i32,
    config: &GameConfig,
    state: &GameState,
    _turn: i32,
    enemy_side: Option<EnemySide>,
) -> Vec<Action> {
    let heroes: Vec<&Hero> = state
        .heroes
        .iter()
        .filter(|h| h.owner_id == player_id)
        .collect();

    let enemies: Vec<&Hero> = state
        .heroes
        .iter()
        .filter(|h| h.owner_id != player_id)
        .collect();

    let mut actions = Vec::new();

    for hero in heroes {
        let allies: Vec<&Hero> = state
            .heroes
            .iter()
            .filter(|h| h.owner_id == player_id && h.id != hero.id)
            .collect();

        let eval_context = EvalContext {
            hero,
            _enemeies: &enemies,
            allies: &allies,
            walls: &state.walls,
            projectiles: &state.projectiles,
            config,
            enemy_side,
        };

        let mut max_score = MIN_SCORE;

        let mut best_action = Action::Move(MoveArgs {
            hero_id: hero.id,
            x: hero.x,
            y: hero.y,
        });

        if hero.cooldown == 0 {
            for enemy in &enemies {
                let score = SHOOT_SCORE;
                if score > max_score {
                    max_score = score;
                    best_action = Action::Shoot(ShootArgs {
                        hero_id: hero.id,
                        x: enemy.x,
                        y: enemy.y,
                    });
                }
            }
        }

        for (dx, dy) in POSSIBLE_MOVES {
            let target_x = hero.x + (dx * 3);
            let target_y = hero.y + (dy * 3);

            if is_valid_move(target_x, target_y, &eval_context) {
                let immediate = eval_pos(target_x, target_y, &eval_context);
                let future = best_reachable_score(target_x, target_y, &eval_context, 3);
                let score = immediate + future;

                if score > max_score {
                    max_score = score;
                    best_action = Action::Move(MoveArgs {
                        hero_id: hero.id,
                        x: target_x,
                        y: target_y,
                    });
                }
            }
        }
        actions.push(best_action);
    }
    actions
}

fn best_reachable_score(tx: i32, ty: i32, ctx: &EvalContext, depth: u32) -> i32 {
    let mut best = i32::MIN;

    for (dx, dy) in POSSIBLE_MOVES {
        let nx = tx + (dx * 3);
        let ny = ty + (dy * 3);

        if is_valid_move(nx, ny, ctx) {
            let immediate = eval_pos(nx, ny, ctx);
            let future = if depth > 0 {
                best_reachable_score(nx, ny, ctx, depth - 1)
            } else {
                0
            };
            let score = immediate + future;
            if score > best {
                best = score;
            }
        }
    }

    if best == i32::MIN { -1000 } else { best }
}

fn is_valid_move(tx: i32, ty: i32, ctx: &EvalContext) -> bool {
    ty < ctx.config.height
        && ty > -1
        && tx < ctx.config.width
        && tx > -1
        && !ctx
            .walls
            .iter()
            .any(|w| (tx - w.x).abs() < 2 && (ty - w.y).abs() < 2)
}

fn eval_pos(tx: i32, ty: i32, ctx: &EvalContext) -> i32 {
    let mut score = 0;

    let enemy_side_y = match ctx.enemy_side {
        Some(side) => match side {
            EnemySide::Bottom => ctx.config.height,
            EnemySide::Top => 0,
        },
        None => 0,
    };

    // move towards the enemy side
    score += (ctx.config.height - (ty - enemy_side_y).abs()) / 2;

    // prevent standing still
    if ctx.hero.x == tx && ctx.hero.y == ty {
        score -= DONT_MOVE_PENALTY;
    }

    // try staying close to ally
    for ally in ctx.allies {
        let dist = (tx - ally.x).abs() + (ty - ally.y).abs();
        if dist <= 30 {
            score += CLOSE_TOGHETER_BONUS;
        }
    }

    if ctx
        .walls
        .iter()
        .any(|w| (tx - w.x).abs() < 2 && (ty - w.y).abs() < 2)
    {
        // do not move into a wall
        score -= MOVE_INTO_WALL_PENALTY;
    }

    for p in ctx.projectiles {
        if (tx - p.x).abs() < 3 && (ty - p.y).abs() < 3 {
            // aslo don t get hit
            score -= HIT_BY_PROJECTILE_PENALTY;
        }
    }

    score
}
