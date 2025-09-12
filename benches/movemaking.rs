mod common;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench(c: &mut Criterion) {
    let board = common::midgame_board();
    let moves = board.generate_all_legal_moves();
    c.bench_function("Make/Unmake all legal moves", |b| {
        b.iter(|| {
            for m in &moves {
                m.make(&board);
            }
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench
}
criterion_main!(benches);
