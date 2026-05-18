pub trait State {
    /// The type of action that can be taken in the state (e.g., tuple of coordinates).
    type Action: Clone;

    /// Returns the default action for the state (used for root node).
    fn default_action(&self) -> Self::Action;

    /// Checks if the specified player has won the game.
    fn player_has_won(&self, player: usize) -> bool;

    /// Determines if the current state is a terminal state (no further moves possible).
    fn is_terminal(&self) -> bool;

    /// Returns a vector of legal actions available from the current state.
    fn get_legal_actions(&self) -> Vec<Self::Action>;

    /// Returns the index of the player whose turn it is to play.
    fn to_play(&self) -> usize;

    /// Returns a new state resulting from applying the given action to the current state.
    fn step(&self, action: Self::Action) -> Self;

    /// Calculates and returns the reward for the specified player in the current state.
    fn reward(&self, player: usize) -> i32;

    /// Renders or prints the current state (useful for debugging or display purposes).
    fn render(&self);
}
