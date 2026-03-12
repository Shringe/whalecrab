use criterion::{BenchmarkId, Criterion, Throughput};
use whalecrab_engine::engine::Engine;
use whalecrab_lib::game::Game;
mod common;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Engine against self");
    let mut sample_engine = Engine::default();

    for depth in 0..=3 {
        let _ = sample_engine.get_engine_move_minimax(depth);
        let sample = sample_engine.nodes_searched;
        group.throughput(Throughput::Elements(sample));

        let mut engine = Engine::default();
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            b.iter(|| {
                if let Some(m) = engine.get_engine_move_minimax(depth) {
                    engine.game.play(&m);
                } else {
                    // Reset the board if no moves to play
                    engine.with_new_game(Game::default());
                }
            });
        });

        println!("Num nodes from starting position: {}", sample);
        println!("Total Nodes Searched: {}", engine.nodes_searched);
    }

    group.finish();
}

criterion::criterion_group! {
    name = benches;
    config = common::configured_criterion().sample_size(10);
    targets = bench
}
criterion::criterion_main!(benches);
