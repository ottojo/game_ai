use game_ai::{GameAi, GameRules};
use hexxagon_lib::game::rules::HexxagonRules;
use mcts::GenericMonteCarloTreeSearchAi;

#[test]
fn test_hexxagon() {
    let initial_state = <HexxagonRules as GameRules>::State::default();

    let mut ai =
        GenericMonteCarloTreeSearchAi::<HexxagonRules>::new(mcts::StopCondition::Iterations(100));
    let _resulting_move = ai.determine_next_move(&initial_state);
}
