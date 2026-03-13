use std::time::Duration;

use criterion::{Criterion, Throughput};
use whalecrab_engine::engine::Engine;
mod common;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Iterative deepening");
    let mut sample_engine = Engine::default();

    for seconds in 0..=3 {
        let duration = Duration::from_secs(seconds);

        sample_engine.nodes_searched = 0;
        let _ = sample_engine.iterative_deepening(&duration);
        let sample = sample_engine.nodes_searched;
        group.throughput(Throughput::Elements(sample));

        let mut engine = Engine::default();
        group.bench_function(
            format!("Iterative deepening for {} seconds", seconds),
            |b| {
                b.iter(|| engine.iterative_deepening(&duration));
            },
        );

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
