use std::time::Duration;

use criterion::{Criterion, Throughput};
use whalecrab_engine::engine::Engine;
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

    let duration = Duration::from_secs(10);
    for depth in [2, 4, 8] {
        let result = engine.search(duration, depth);
        group.throughput(Throughput::Elements(result.nodes));

        println!("{}", format_header(&format!(" depth of {} ", depth)));
        println!("Nodes searched:   {}", result.nodes);
        println!("Depth reached:    {}", result.depth);
        println!("Final score:      {}", result.score);
        println!("Termination:      {:?}", result.terminal);
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
