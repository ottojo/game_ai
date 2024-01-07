use criterion::{criterion_group, criterion_main, Criterion};
use hexxagonlib::{ai::random_rollout, game::GameState};

fn criterion_benchmark(c: &mut Criterion) {
    let initial_state = GameState::initialize();
    c.bench_function("random_rollout", |b| {
        b.iter(|| random_rollout(&initial_state))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
