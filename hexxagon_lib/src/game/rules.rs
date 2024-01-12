use crate::ai::{
    move_generation::{self, sample_valid_move},
    HexxagonMove,
};

use game_ai::{GameRules, GameStateTrait, PlayerIndex, Rewards};

use super::{GameResult, GameState, Player};

#[derive(Clone)]
pub struct HexxagonRules {}

impl GameRules for HexxagonRules {
    type State = GameState;
    type Action = HexxagonMove;

    const N_PLAYERS: u32 = 2;

    fn play(initial_state: &Self::State, action: &Self::Action) -> Self::State {
        let mut new_state = initial_state.clone();
        new_state.player_move(action.src, action.dst);
        new_state
    }

    fn random_rollout(initial_state: &Self::State) -> Rewards {
        let mut state = initial_state.clone();
        while !state.is_final() {
            let random_move = sample_valid_move(&state);
            state.player_move(random_move.src, random_move.dst);
        }

        state.reward()
    }
}

impl GameStateTrait<HexxagonMove> for GameState {
    fn is_final(&self) -> bool {
        self.result().is_some()
    }

    fn get_actions(&self) -> Vec<HexxagonMove> {
        move_generation::all_moves(self)
    }

    fn reward(&self) -> Rewards {
        match self.result() {
            Some(GameResult::Tie) => Rewards {
                player_0: 0.5,
                player_1: 0.5,
            },
            Some(GameResult::Win(Player::Rubies)) => Rewards {
                player_0: 1.0,
                player_1: 0.0,
            },
            Some(GameResult::Win(Player::Pearls)) => Rewards {
                player_0: 0.0,
                player_1: 1.0,
            },
            None => panic!(),
        }
    }

    fn next_player(&self) -> PlayerIndex {
        match self.next_player() {
            Player::Rubies => PlayerIndex::Zero,
            Player::Pearls => PlayerIndex::One,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::initialize()
    }
}
