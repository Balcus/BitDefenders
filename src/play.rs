use crate::types::{EnemySide, GameConfig, GameState, Hero, MoveArgs, Projectile, ShootArgs, Wall};

#[derive(Debug, Clone)]
pub enum Action {
    Move(MoveArgs),
    Shoot(ShootArgs),
}

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
        let mut max_score = i32::MIN;

        let mut best_action = Action::Move(MoveArgs {
            hero_id: hero.id,
            x: hero.x,
            y: hero.y,
        });

        if hero.cooldown == 0 {
            for _enemy in &enemies {
                // TODO: soothing
            }
        }

        for (dx, dy) in POSSIBLE_MOVES {
            let target_x = hero.x + (dx * 3);
            let target_y = hero.y + (dy * 3);

            if is_valid_move(target_x, target_y, config, &state.walls) {
                let immediate = eval_pos(
                    target_x,
                    target_y,
                    hero,
                    &enemies,
                    &state.walls,
                    &state.projectiles,
                    config,
                    enemy_side,
                );

                let future = best_reachable_score(
                    target_x,
                    target_y,
                    hero,
                    &enemies,
                    &state.walls,
                    &state.projectiles,
                    config,
                    enemy_side,
                    3,
                );

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

fn best_reachable_score(
    tx: i32,
    ty: i32,
    hero: &Hero,
    enemies: &Vec<&Hero>,
    walls: &Vec<Wall>,
    projectiles: &Vec<Projectile>,
    config: &GameConfig,
    enemy_side: Option<EnemySide>,
    depth: u32,
) -> i32 {
    let mut best = i32::MIN;

    for (dx, dy) in POSSIBLE_MOVES {
        let nx = tx + (dx * 3);
        let ny = ty + (dy * 3);

        if is_valid_move(nx, ny, config, walls) {
            let immediate = eval_pos(
                nx,
                ny,
                hero,
                enemies,
                walls,
                projectiles,
                config,
                enemy_side,
            );
            let future = if depth > 0 {
                best_reachable_score(
                    nx,
                    ny,
                    hero,
                    enemies,
                    walls,
                    projectiles,
                    config,
                    enemy_side,
                    depth - 1,
                )
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

fn is_valid_move(tx: i32, ty: i32, config: &GameConfig, walls: &Vec<Wall>) -> bool {
    ty < config.height
        && ty > -1
        && tx < config.width
        && tx > -1
        && !walls
            .iter()
            .any(|w| (tx - w.x).abs() < 2 && (ty - w.y).abs() < 2)
}

fn eval_pos(
    tx: i32,
    ty: i32,
    hero: &Hero,
    _enemies: &Vec<&Hero>,
    walls: &Vec<Wall>,
    projectiles: &Vec<Projectile>,
    config: &GameConfig,
    enemy_side: Option<EnemySide>,
) -> i32 {
    let mut score = 0;

    // this needs to be calculated only once at the start of the round, else they will just be stuck in the middle...
    let enemy_side_y = match enemy_side {
        Some(side) => match side {
            EnemySide::Bottom => config.height,
            EnemySide::Top => 0,
        },
        None => 0,
    };

    score += (config.height - (ty - enemy_side_y).abs()) * 10;

    if hero.x == tx && hero.y == ty {
        score -= 10;
    }

    if walls
        .iter()
        .any(|w| (tx - w.x).abs() < 2 && (ty - w.y).abs() < 2)
    {
        // do not move into a wall
        score -= 500;
    }

    for p in projectiles {
        if (tx - p.x).abs() < 3 && (ty - p.y).abs() < 3 {
            // aslo don t get hit
            score -= 500;
        }
    }

    score
}
