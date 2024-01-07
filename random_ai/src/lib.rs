use rand::seq::SliceRandom;

use game_ai::GameAi;
use game_ai::GameRules;
use game_ai::GameStateTrait;

#[derive(Clone)]
pub struct RandomAi {}

impl<Rules: GameRules> GameAi<Rules> for RandomAi {
    fn determine_next_move(&mut self, gamestate: &Rules::State) -> Rules::Action {
        let possible_moves = gamestate.get_actions();
        assert!(!possible_moves.is_empty());

        possible_moves
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone()
    }

    fn name(&self) -> String {
        "RandomAi".to_owned()
    }
}
