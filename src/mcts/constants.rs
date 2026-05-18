pub const POSSIBLE_MOVES: [(i32, i32); 9] = [
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

pub const POSSIBLE_SHOOT_MOVES: [(i32, i32); 8] = [
    (0, 1),
    (0, -1),
    (1, 0),
    (-1, 0),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

pub const MOVE_INTO_WALL_PENALTY: i32 = 25;
pub const HIT_BY_PROJECTILE_PENALTY: i32 = 500;
pub const DONT_MOVE_PENALTY: i32 = 25;
pub const SHOOT_SCORE: i32 = 450;
pub const CLOSE_TOGETHER_BONUS: i32 = 30;
pub const EXPLORATION_BONUS: i32 = 50;
pub const ENEMY_SEES_U_PENALTY: i32 = 40;
pub const APPROACH_ENEMY_WEIGHT: i32 = 1;
