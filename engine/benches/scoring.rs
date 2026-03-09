mod common;
use criterion::Criterion;

fn bench(c: &mut Criterion) {
    let mut engine = common::midgame();
    c.bench_function(
        "Scoring middle game position with transposition table",
        |b| b.iter(|| engine.grade_position()),
    );

    let mut engine = common::midgame();
    c.bench_function("Scoring middle game position", |b| {
        b.iter(|| {
            engine.clear_persistant_cache();
            engine.grade_position()
        })
    });

    let mut engine = common::earlygame();
    c.bench_function("Scoring early game position", |b| {
        b.iter(|| {
            engine.clear_persistant_cache();
            engine.grade_position()
        })
    });

    let mut engine = common::lategame();
    c.bench_function("Scoring late game position", |b| {
        b.iter(|| {
            engine.clear_persistant_cache();
            engine.grade_position()
        })
    });
}

setup_criterion!();
