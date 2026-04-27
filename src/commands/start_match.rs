use serde::Deserialize;

use crate::commands::shared::{GameConfig, GameState};

#[derive(Deserialize, Debug)]
pub struct StartMatch {
    pub match_id: String,
    pub your_player_id: i32,
    pub config: GameConfig,
    pub state: GameState
}