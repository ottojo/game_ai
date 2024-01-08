use criterion::{criterion_group, criterion_main, Criterion};
use game_ai::{GameAi, GameRules};
use hexxagon_lib::game::rules::HexxagonRules;
use mcts::GenericMonteCarloTreeSearchAi;
use tic_tac_toe::TTTRules;

fn tic_tac_toe_search(c: &mut Criterion) {
    let initial_state = <TTTRules as GameRules>::State::default();
    c.bench_function("ttt_100_iterations", |b| {
        b.iter(|| {
            let mut ai = GenericMonteCarloTreeSearchAi::<TTTRules>::new(
                mcts::StopCondition::Iterations(100),
            );
            ai.determine_next_move(&initial_state)
        })
    });
}

fn hexxagon_search(c: &mut Criterion) {
    let initial_state = <HexxagonRules as GameRules>::State::default();
    c.bench_function("hexxagon_100_iterations", |b| {
        b.iter(|| {
            let mut ai = GenericMonteCarloTreeSearchAi::<HexxagonRules>::new(
                mcts::StopCondition::Iterations(100),
            );
            ai.determine_next_move(&initial_state)
        })
    });
}

criterion_group!(benches, tic_tac_toe_search, hexxagon_search);
criterion_main!(benches);
