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
    let mut sample_engine = Engine::default();

    for seconds in 1..=10 {
        let duration = Duration::from_secs(seconds);

        sample_engine.nodes_searched = 0;
        let result = sample_engine.search(duration, u16::MAX);
        let sample = sample_engine.nodes_searched;
        group.throughput(Throughput::Elements(sample));

        println!("{}", format_header(&format!(" {} seconds ", seconds)));
        println!("Nodes searched:   {}", sample);
        println!("Depth reached:    {}", result.depth);
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
