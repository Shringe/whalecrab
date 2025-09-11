use criterion::{criterion_group, criterion_main, Criterion};
use whalecrab::board::Board;

fn engine_vs_engine(board: &mut Board) {
    *board = if let Some(m) = board.get_engine_move_minimax(3) {
        println!("Playing: {}", m);
        m.make(board)
    } else {
        println!("Ran out of moves, resetting board.");
        Board::default()
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut board = Board::default();
    c.bench_function("Evaluation: engine_vs_engine", |b| {
        b.iter(|| engine_vs_engine(&mut board))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
