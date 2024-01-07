use std::{fs, time::Duration};

use game_ai::{GameRules, GameStateTrait, PlayerIndex, Rewards};
use graphviz_rust::printer::{DotPrinter, PrinterContext};
use mcts::GenericMonteCarloTreeSearchAi;
use rand::seq::SliceRandom;
use tic_tac_toe::{TTTPlayer, TTTRules, TTTState};

fn play_against_random() -> game_ai::Rewards {
    let mut game_state = TTTState::default();
    let ai = GenericMonteCarloTreeSearchAi::new(Duration::from_millis(100), TTTRules {});
    let mut i = 0;
    while !game_state.is_final() {
        println!("{}", game_state);
        println!("Possible moves: {:?}", game_state.get_actions());
        if game_state.next_player() == PlayerIndex::from(TTTPlayer::X) {
            let (ai_move, debug) = ai.best_action(game_state.clone());
            fs::write(
                format!("{:03}.dot", i),
                debug.dot_graph.print(&mut PrinterContext::default()),
            )
            .unwrap_or_else(|_| println!("Error writing file!"));
            println!("AI move: {:?}", &ai_move);
            game_state = TTTRules::play(&game_state, &ai_move);
        } else {
            let possible_moves = game_state.get_actions();
            let random_move = possible_moves.choose(&mut rand::thread_rng()).unwrap();
            println!("Random move: {:?}", random_move);
            game_state = TTTRules::play(&game_state, random_move);
        }
        i += 1;
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
