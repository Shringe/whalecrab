mod common;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench(c: &mut Criterion) {
    let mut board = common::midgame_board();
    c.bench_function(
        "Scoring middle game position with transposition table",
        |b| b.iter(|| board.grade_position()),
    );

    let mut board = common::midgame_board();
    c.bench_function("Scoring middle game position", |b| {
        b.iter(|| {
            board.transposition_table.clear();
            board.grade_position()
        })
    });

    let mut board = common::earlygame_board();
    c.bench_function("Scoring early game position", |b| {
        b.iter(|| {
            board.transposition_table.clear();
            board.grade_position()
        })
    });

    let mut board = common::lategame_board();
    c.bench_function("Scoring late game position", |b| {
        b.iter(|| {
            board.transposition_table.clear();
            board.grade_position()
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench
}
criterion_main!(benches);
