use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput};
use whalecrab_engine::{engine::Engine, timers::infinite::Infinite};
use whalecrab_lib::position::game::Game;
mod common;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Engine against self with minimax");
    let mut sample_engine = Engine::default();

    for depth in 1..=4 {
        let result = sample_engine.minimax(&Infinite, depth);
        let sample = result.info.nodes;
        group.throughput(Throughput::Elements(sample));

        let mut engine = Engine::default();
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            b.iter(|| {
                if let Some(m) = engine.minimax(&Infinite, depth).best_move {
                    engine.game.play(&m);
                } else {
                    // Reset the board if no moves to play
                    engine.with_new_game(Game::default());
                }
            });
        });
    }

    group.finish();
}

criterion::criterion_group! {
    name = benches;
    config = common::configured_criterion().sample_size(10).measurement_time(Duration::from_secs(20));
    targets = bench
}
criterion::criterion_main!(benches);
