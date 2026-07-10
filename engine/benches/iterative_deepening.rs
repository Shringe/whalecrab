use std::time::Duration;

use criterion::{Criterion, Throughput};
use whalecrab_engine::{engine::Engine, timers::elapsed::Elapsed};
mod common;

fn format_header(title: &str) -> String {
    let width: usize = 30;
    if title.is_empty() {
        return "=".repeat(width);
    }

    let padding = width.saturating_sub(title.len());
    let left = padding / 2;
    let right = padding - left;
    format!("{:=<left$}{}{:=>right$}", "", title, "")
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Iterative deepening");
    let mut engine = Engine::default();

    let threads = 8;

    let duration = Duration::from_secs(20);
    for depth in [2, 4, 6, 8] {
        let timer = Elapsed::now(duration);
        let result = engine.threaded_search(&timer, depth, threads);
        group.throughput(Throughput::Elements(result.nodes));

        println!("{}", format_header(&format!(" depth of {} ", depth)));
        println!("Nodes searched:   {}", result.nodes);
        println!("Depth reached:    {}", result.depth);
        println!("Final score:      {}", result.score);
        println!("Termination:      {:?}", result.terminal);
        println!("{}", timer);
        println!("{}", format_header(""));
    }

    group.finish();
}

criterion::criterion_group! {
    name = benches;
    config = common::configured_criterion().sample_size(10);
    targets = bench
}
criterion::criterion_main!(benches);
