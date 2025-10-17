use criterion::{criterion_group, criterion_main, Criterion};
use whalecrab_lib::game::Game;

fn bench(c: &mut Criterion) {
    let mut game = Game::default();
    c.bench_function("Engine against self", |b| {
        b.iter(|| {
            if let Some(m) = game.get_engine_move_minimax(20) {
                game.play(&m);
            } else {
                // Reset the board if no moves to play
                println!(
                    "Game ended in {:?}. {} nodes searched.",
                    game.position.state, game.nodes_seached
                );
                game = Game::default();
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
