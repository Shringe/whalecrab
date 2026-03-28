mod common;
use std::time::{Duration, Instant};

use criterion::Criterion;
use whalecrab_engine::timers::{elapsed::Elapsed, move_timer::MoveTimer};

#[track_caller]
fn bench_timer<T: MoveTimer>(c: &mut Criterion, timer: &T, id: &str) {
    c.bench_function(id, |b| b.iter(|| timer.over()));
}

fn bench(c: &mut Criterion) {
    let start = Instant::now();
    bench_timer(
        c,
        &Elapsed::new(start, Duration::from_secs(6)),
        "Elapsed timer 6 second",
    );

    bench_timer(
        c,
        &Elapsed::new(start, Duration::MAX),
        "Elapsed timer infinite",
    );

    bench_timer(
        c,
        &Elapsed::new(start, Duration::from_secs(0)),
        "Elapsed timer finished",
    );
}

setup_criterion!(Criterion::default());
