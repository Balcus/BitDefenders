use serde::Deserialize;

use crate::commands::shared::GameState;

#[derive(Deserialize, Debug)]
pub struct StartTurn {
    pub turn: u32,
    pub state: GameState
}