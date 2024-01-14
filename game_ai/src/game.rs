use rand::seq::SliceRandom;

pub trait GameStateTrait<Action>: Default + Clone + core::fmt::Debug {
    fn is_final(&self) -> bool;

    /// All possible actions for the next player from this state
    fn get_actions(&self) -> Vec<Action>;

    /// Reward for each player, if this is a final state
    fn reward(&self) -> Rewards;

    fn next_player(&self) -> PlayerIndex;

    /// Player which made the move resulting in this state
    fn incoming_player(&self) -> PlayerIndex {
        self.next_player().opponent()
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Rewards {
    pub player_0: f32,
    pub player_1: f32,
}

impl Rewards {
    pub fn for_player(&self, p: &PlayerIndex) -> f32 {
        match p {
            PlayerIndex::Zero => self.player_0,
            PlayerIndex::One => self.player_1,
        }
    }
}

impl std::ops::AddAssign<&Self> for Rewards {
    fn add_assign(&mut self, rhs: &Self) {
        self.player_0 += rhs.player_0;
        self.player_1 += rhs.player_1;
    }
}

#[derive(PartialEq)]
pub enum PlayerIndex {
    Zero,
    One,
}
impl From<PlayerIndex> for usize {
    fn from(val: PlayerIndex) -> Self {
        match val {
            PlayerIndex::Zero => 0,
            PlayerIndex::One => 1,
        }
    }
}

impl PlayerIndex {
    pub fn opponent(&self) -> PlayerIndex {
        match self {
            PlayerIndex::Zero => PlayerIndex::One,
            PlayerIndex::One => PlayerIndex::Zero,
        }
    }

    pub fn is_maximizing(&self) -> bool {
        match self {
            PlayerIndex::Zero => true,
            PlayerIndex::One => false,
        }
    }
}

pub trait GameRules {
    type Action: core::fmt::Debug + std::hash::Hash + Eq + Clone;
    type State: GameStateTrait<Self::Action>;
    const N_PLAYERS: u32;

    fn play(initial_state: &Self::State, action: &Self::Action) -> Self::State;

    // Play game until completion from a given state
    fn random_rollout(initial_state: &Self::State) -> Rewards {
        let mut state: Self::State = initial_state.clone();
        while !state.is_final() {
            let all_possible_actions = state.get_actions();
            let random_action = all_possible_actions
                .choose(&mut rand::thread_rng())
                .expect("Rollout failed, no actions possible!");
            state = Self::play(&state, random_action);
        }

        state.reward()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused)]
    fn do_stuff_with_game<Rules: GameRules>() {
        let initial_game_state = Rules::State::default();
        let moves = initial_game_state.get_actions();
        let some_move = &moves[0];
        let second_state = Rules::play(&initial_game_state, some_move);
        if second_state.is_final() {
            println!(
                "Move {:?} ended game with reward {:?}!",
                some_move,
                second_state.reward()
            );
        } else {
            println!("Move did not end game.");
        }
    }
}
