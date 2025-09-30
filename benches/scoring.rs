mod common;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench(c: &mut Criterion) {
    let mut game = common::midgame();
    c.bench_function(
        "Scoring middle game position with transposition table",
        |b| b.iter(|| game.grade_position()),
    );

    let mut game = common::midgame();
    c.bench_function("Scoring middle game position", |b| {
        b.iter(|| {
            game.transposition_table.clear();
            game.grade_position()
        })
    });

    let mut board = common::earlygame();
    c.bench_function("Scoring early game position", |b| {
        b.iter(|| {
            board.transposition_table.clear();
            board.grade_position()
        })
    });

    let mut game = common::lategame();
    c.bench_function("Scoring late game position", |b| {
        b.iter(|| {
            game.transposition_table.clear();
            game.grade_position()
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench
}
criterion_main!(benches);
