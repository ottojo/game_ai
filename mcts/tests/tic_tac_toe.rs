use game_ai::{GameAi, GameRules, GameStateTrait, PlayerIndex, Rewards};

use mcts::{GenericMonteCarloTreeSearchAi, StopCondition};
use rand::seq::SliceRandom;

use tic_tac_toe::{TTTPlayer, TTTRules, TTTState};

/*
use std::fs;
use graphviz_rust::printer::{DotPrinter, PrinterContext};
*/

fn play_against_random() -> game_ai::Rewards {
    let mut game_state = TTTState::default();
    let mut ai = GenericMonteCarloTreeSearchAi::<TTTRules>::new(StopCondition::Iterations(10000));
    // let mut i = 0;
    while !game_state.is_final() {
        println!("{}", game_state);
        println!("Possible moves: {:?}", game_state.get_actions());
        if game_state.next_player() == PlayerIndex::from(TTTPlayer::X) {
            let ai_move = ai.determine_next_move(&game_state);
            /*
                let debug_data = ai.get_last_graphviz();
                fs::write(
                    format!("{:03}.dot", i),
                    debug_data.print(&mut PrinterContext::default()),
                )
                .unwrap_or_else(|_| println!("Error writing file!"));
            */
            println!("AI move: {:?}", &ai_move);
            game_state = TTTRules::play(&game_state, &ai_move);
        } else {
            let possible_moves = game_state.get_actions();
            let random_move = possible_moves.choose(&mut rand::thread_rng()).unwrap();
            println!("Random move: {:?}", random_move);
            game_state = TTTRules::play(&game_state, random_move);
        }
        // i += 1;
    }

    game_state.reward()
}

#[test]
fn test_generic_mcts_tic_tac_toe() {
    for _ in 0..100 {
        assert_ne!(
            play_against_random(),
            Rewards {
                player_0: 0.0,
                player_1: 1.0
            },
            "MCTS must not loose against random opponent in tic tac toe"
        )
    }
}
