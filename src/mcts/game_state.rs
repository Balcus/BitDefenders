use crate::{
    grid::Grid,
    mcts::{
        constants::{self, ENEMY_SEES_U_PENALTY, HIT_BY_PROJECTILE_PENALTY},
        state::State,
        utils::{self, has_line_of_sight, update_bullets, update_movement},
    },
    types::{self, Hero, MoveArgs, Projectile, ShootArgs},
};

#[derive(Clone)]
pub struct GameState {
    pub player_id: i32,
    pub hero_id: i32,
    pub state: types::GameState,
    pub config: types::GameConfig,
    pub grid: Grid,
    // pub turn: i32,
}

impl GameState {
    pub fn new(
        player_id: i32,
        hero_id: i32,
        state: types::GameState,
        config: types::GameConfig,
        _turn: i32,
    ) -> Self {
        let grid = Grid::from(config.width, config.height, &state.walls.clone());
        Self {
            player_id,
            hero_id,
            state,
            config,
            grid,
            // turn,
        }
    }

    fn me(&self) -> &Hero {
        self.state
            .heroes
            .iter()
            .find(|h| h.id == self.hero_id)
            .unwrap()
    }

    fn other(&self) -> &Hero {
        self.state
            .heroes
            .iter()
            .find(|h| h.id != self.hero_id)
            .unwrap()
    }

    fn enemies(&self) -> Vec<&Hero> {
        self.state
            .heroes
            .iter()
            .filter(|h| h.owner_id != self.player_id)
            .collect()
    }

    fn is_hit(&self) -> bool {
        self.state.projectiles.iter().any(|b| {
            b.owner_id != self.player_id
                && (self.me().x - b.x).abs() < 3
                && (self.me().y - b.y).abs() < 3
        })
    }
}

impl State for GameState {
    type Action = types::Action;

    fn default_action(&self) -> Self::Action {
        Self::Action::Move(MoveArgs {
            hero_id: self.hero_id,
            x: self.me().x,
            y: self.me().y,
            comment: None,
        })
    }

    fn player_has_won(&self, player: usize) -> bool {
        todo!()
    }

    fn is_terminal(&self) -> bool {
        todo!()
    }

    fn get_legal_actions(&self) -> Vec<Self::Action> {
        let mut actions = Vec::new();

        if self.me().cooldown == 0 {
            for enemy in self.enemies() {
                if utils::has_line_of_sight(
                    self.me().x,
                    self.me().y,
                    enemy.x,
                    enemy.y,
                    &self.state.walls,
                ) {
                    for (dx, dy) in constants::POSSIBLE_SHOOT_MOVES {
                        actions.push(Self::Action::Move(MoveArgs {
                            hero_id: self.hero_id,
                            x: dx,
                            y: dy,
                            comment: Some(String::from("Phew 💥")),
                        }));
                    }
                }
            }
        }

        for (dx, dy) in constants::POSSIBLE_MOVES {
            let target_x = self.me().x + (dx * 3);
            let target_y = self.me().y + (dy * 3);

            if utils::is_valid_move(target_x, target_y, &self.config, &self.state.walls) {
                actions.push(Self::Action::Move(MoveArgs {
                    hero_id: self.hero_id,
                    x: target_x,
                    y: target_y,
                    comment: Some(String::from("Moving 🏃‍♂️")),
                }));
            }
        }

        actions
    }

    fn to_play(&self) -> usize {
        todo!()
    }

    fn step(&self, action: Self::Action) -> Self {
        match action {
            Self::Action::Move(args) => {
                let mut new_state = self.clone();
                update_bullets(&mut new_state.state);
                update_movement(&mut new_state.state, self.hero_id, args.x, args.y);
                new_state
            }
            types::Action::Shoot(args) => {
                let mut new_state = self.clone();
                update_bullets(&mut new_state.state);
                new_state.state.projectiles.push(Projectile {
                    owner_id: self.player_id,
                    type_: String::from("bullet"),
                    origin_x: self.me().x,
                    origin_y: self.me().y,
                    x: args.x,
                    y: args.y,
                    ttl: self.config.hero_types.get("sniper").unwrap().projectile_ttl,
                });
                new_state
            }
        }
    }

    fn reward(&self, to_play: usize) -> i32 {
        let mut reward = i32::MIN;
        if self.is_hit() {
            reward -= HIT_BY_PROJECTILE_PENALTY;
        }

        for enemy in self.enemies() {
            if has_line_of_sight(
                self.me().x,
                self.me().y,
                enemy.x,
                enemy.y,
                &self.state.walls,
            ) {
                reward -= ENEMY_SEES_U_PENALTY;
            }
        }
        reward
    }

    fn render(&self) {
        todo!()
    }
}
