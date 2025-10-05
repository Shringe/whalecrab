mod common;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench(c: &mut Criterion) {
    let mut game = common::midgame();
    let moves = game.generate_all_legal_moves();
    c.bench_function("Make/Unmake all legal moves", |b| {
        b.iter(|| {
            for m in &moves {
                game.play(&m);
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
