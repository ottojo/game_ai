use std::time::Duration;

use game_ai::GameAi;
use hexxagon_lib::game::{self, rules::HexxagonRules, GameResult, GameState, MoveResult, Player};
use indicatif::ProgressIterator;
use mcts::GenericMonteCarloTreeSearchAi;
use random_ai::RandomAi;

fn play_game<PearlsAI: GameAi<HexxagonRules>, RubiesAI: GameAi<HexxagonRules>>(
    mut pearls_ai: PearlsAI,
    mut rubies_ai: RubiesAI,
) -> GameResult {
    let mut gamestate = GameState::initialize();
    let mut i = 0;
    while gamestate.result().is_none() {
        let ai: &mut dyn GameAi<HexxagonRules> = match gamestate.next_player() {
            game::Player::Rubies => &mut rubies_ai,
            game::Player::Pearls => &mut pearls_ai,
        };
        let ai_move = ai.determine_next_move(&gamestate);
        let result = gamestate.player_move(ai_move.src, ai_move.dst);
        assert_eq!(result, MoveResult::Success);
        i += 1;
    }
    println!("Game ended after {} moves", i);
    gamestate.result().unwrap()
}

fn main() {
    let mut pearls_wins = 0;
    let mut rubies_wins = 0;

    let create_pearls_ai = || {
        GenericMonteCarloTreeSearchAi::<HexxagonRules>::new(mcts::StopCondition::Time(
            Duration::from_millis(100),
        ))
    };
    let create_rubies_ai = || RandomAi {};

    for _i in (0..20).progress() {
        let game_result = play_game(create_pearls_ai(), create_rubies_ai());
        match game_result {
            GameResult::Tie => {}
            GameResult::Win(Player::Rubies) => rubies_wins += 1,
            GameResult::Win(Player::Pearls) => pearls_wins += 1,
        }
    }

    println!(
        "Rubies ({}) wins: {}",
        GameAi::<HexxagonRules>::name(&create_rubies_ai()),
        rubies_wins
    );
    println!(
        "Pearls ({}) wins: {}",
        GameAi::name(&create_pearls_ai()),
        pearls_wins
    );
}
