use criterion::{BenchmarkId, Criterion, Throughput};
use whalecrab_engine::engine::Engine;
use whalecrab_lib::game::Game;
mod common;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Engine against self");
    // let mut averages = Vec::new();
    let mut sample_engine = Engine::default();

    for depth in 0..=0 {
        let mut nodes_searched = 0;
        let mut num_completions = 0;

        let _ = sample_engine.get_engine_move_minimax(depth);
        let sample = sample_engine.game.nodes_seached;

        group.throughput(Throughput::Elements(sample.try_into().unwrap_or(u64::MAX)));
        println!("{}", sample);
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            let mut engine = Engine::default();
            b.iter(|| {
                if let Some(m) = engine.get_engine_move_minimax(depth) {
                    engine.game.play(&m);
                } else {
                    // Reset the board if no moves to play
                    engine.with_new_game(Game::default());
                }
            });

            nodes_searched += engine.game.nodes_seached;
            num_completions += 1;
        });

        let average = nodes_searched / num_completions;
        println!("Nodes Searched: {}", average);
    }

    group.finish();
}

criterion::criterion_group! {
    name = benches;
    config = common::configured_criterion().sample_size(10);
    targets = bench
}
criterion::criterion_main!(benches);
