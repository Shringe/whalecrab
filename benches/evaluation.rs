use criterion::{criterion_group, criterion_main, Criterion};
use whalecrab::board::Board;

fn bench(c: &mut Criterion) {
    let mut board = Board::default();
    c.bench_function("Engine against self", |b| {
        b.iter(|| {
            board = if let Some(m) = board.get_engine_move_minimax(3) {
                m.make(&board)
            } else {
                // Reset the board if no moves to play
                Board::default()
            }
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench
}
criterion_main!(benches);
