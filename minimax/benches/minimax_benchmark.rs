use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
};
use game_ai::{GameAi, GameRules};
use hexxagon_lib::{ai::HexxagonEvaluator, game::rules::HexxagonRules};
use minimax::MiniMax;

fn hexxagon_search(c: &mut Criterion) {
    let initial_state = <HexxagonRules as GameRules>::State::default();

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("hexxagon_minimax");
    group.plot_config(plot_config);

    for depth in 1usize..=3 {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            b.iter(|| {
                let mut ai = MiniMax::new(depth, HexxagonEvaluator {});
                ai.determine_next_move(&initial_state)
            });
        });
    }
    group.finish();
}

criterion_group!(benches, hexxagon_search);
criterion_main!(benches);
