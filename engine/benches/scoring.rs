mod common;
use criterion::Criterion;
use whalecrab_engine::engine::TRANSPOSITION_TABLE_MEMORY_BUDGET_IN_KILOBYTES;

fn bench(c: &mut Criterion) {
    let _ = TRANSPOSITION_TABLE_MEMORY_BUDGET_IN_KILOBYTES.set(1);

    let mut engine = common::midgame();
    let mut dummy = engine.clone();
    c.bench_function("Scoring middle game with transposition table", |b| {
        b.iter(|| {
            dummy.clear_persistant_cache();
            engine.grade_position()
        })
    });

    let mut engine = common::midgame();
    c.bench_function("Scoring middle game", |b| {
        b.iter(|| {
            engine.clear_persistant_cache();
            engine.grade_position()
        })
    });

    let mut engine = common::earlygame();
    let mut dummy = engine.clone();
    c.bench_function("Scoring early game with transposition table", |b| {
        b.iter(|| {
            dummy.clear_persistant_cache();
            engine.grade_position()
        })
    });

    let mut engine = common::earlygame();
    c.bench_function("Scoring early game", |b| {
        b.iter(|| {
            engine.clear_persistant_cache();
            engine.grade_position()
        })
    });

    let mut engine = common::lategame();
    let mut dummy = engine.clone();
    c.bench_function("Scoring late game with transposition table", |b| {
        b.iter(|| {
            dummy.clear_persistant_cache();
            engine.grade_position()
        })
    });

    let mut engine = common::lategame();
    c.bench_function("Scoring late game", |b| {
        b.iter(|| {
            engine.clear_persistant_cache();
            engine.grade_position()
        })
    });
}

setup_criterion!(common::configured_criterion());
