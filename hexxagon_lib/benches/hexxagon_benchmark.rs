use criterion::{criterion_group, criterion_main, Criterion};
use game_ai::GameRules;
use hexxagon_lib::game::{rules::HexxagonRules, GameState};

fn hexxagon_rollout(c: &mut Criterion) {
    let initial_state = GameState::default();
    c.bench_function("hexxagon_rollout", |b| {
        b.iter(|| <HexxagonRules as GameRules>::random_rollout(&initial_state))
    });
}

criterion_group!(benches, hexxagon_rollout);
criterion_main!(benches);
