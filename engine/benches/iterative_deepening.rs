use std::time::Duration;

use criterion::{Criterion, Throughput};
use whalecrab_engine::{engine::Engine, move_result::Terminal, timers::elapsed::Elapsed};
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

    let threads = std::thread::available_parallelism().unwrap().into();
    let duration = Duration::from_secs(20);

    for threads in [1, threads] {
        engine.clear_persistant_cache();
        engine.set_threads(threads);
        println!();
        for depth in [2, 4, 6, 7] {
            let timer = Elapsed::now(duration);
            let result = engine.search(&timer, depth);
            group.throughput(Throughput::Elements(result.nodes));

            println!(
                "{}",
                format_header(&format!(" threads: {}; depth: {} ", threads, depth))
            );
            println!("Nodes searched:   {}", result.nodes);
            println!("Depth reached:    {}", result.depth);
            println!("Final score:      {}", result.score);
            println!("Termination:      {:?}", result.terminal);
            println!("{}", timer);
            println!("{}", format_header(""));

            if result.terminal == Terminal::Timer {
                break;
            }
        }
    }

    group.finish();
}

criterion::criterion_group! {
    name = benches;
    config = common::configured_criterion().sample_size(10);
    targets = bench
}
criterion::criterion_main!(benches);
