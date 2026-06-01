pub mod constants;
pub mod eval;
pub mod geometry;
pub mod grid;

use crate::{
    ai::grid::Grid,
    types::{Action, EnemySide, GameConfig, GameState, Hero, MoveArgs, ShootArgs},
};
use constants::LOOKAHEAD_DEPTH;
use eval::{EvalContext, best_reachable_score, eval_pos, is_valid_move, shoot_score};
use geometry::has_line_of_sight;

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
    grid: &mut Grid,
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

    for hero in &heroes {
        let allies: Vec<&Hero> = state
            .heroes
            .iter()
            .filter(|h| h.owner_id == player_id && h.id != hero.id)
            .collect();

        let ctx = EvalContext::new(
            hero,
            &enemies,
            &allies,
            &state.walls,
            &state.projectiles,
            config,
            enemy_side,
            grid,
        );

        let action = choose_action(hero, &enemies, &ctx, state);

        if let Action::Move(ref args) = action {
            if args.x >= 0 && args.x < config.width && args.y >= 0 && args.y < config.height {
                grid.tiles[args.x as usize][args.y as usize].should_consider = false;
            }
        }

        actions.push(action);
    }

    actions
}

fn choose_action<'a>(
    hero: &'a Hero,
    enemies: &[&'a Hero],
    ctx: &EvalContext<'a>,
    state: &GameState,
) -> Action {
    let mut max_score = i32::MIN;
    let mut best_action = Action::Move(MoveArgs {
        hero_id: hero.id,
        x: hero.x,
        y: hero.y,
        comment: None,
    });

    if hero.cooldown == 0 {
        let best_shot = enemies
            .iter()
            .filter(|e| has_line_of_sight(hero.x, hero.y, e.x, e.y, &state.walls))
            .max_by_key(|e| shoot_score(hero, e, ctx.config));

        if let Some(target) = best_shot {
            let score = shoot_score(hero, target, ctx.config);
            if score > max_score {
                max_score = score;
                best_action = Action::Shoot(ShootArgs {
                    hero_id: hero.id,
                    x: target.x,
                    y: target.y,
                    comment: Some(format!("💥 h{} hp:{}", target.id, target.hp)),
                });
            }
        }
    }

    for &(dx, dy) in &POSSIBLE_MOVES {
        let tx = hero.x + dx * 3;
        let ty = hero.y + dy * 3;

        if !is_valid_move(tx, ty, ctx) {
            continue;
        }

        let score = eval_pos(tx, ty, ctx) + best_reachable_score(tx, ty, ctx, LOOKAHEAD_DEPTH);

        if score > max_score {
            max_score = score;
            best_action = Action::Move(MoveArgs {
                hero_id: hero.id,
                x: tx,
                y: ty,
                comment: None,
            });
        }
    }

    best_action
}
