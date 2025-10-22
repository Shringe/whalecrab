use criterion::{Criterion, criterion_group, criterion_main};
use whalecrab_lib::game::Game;

fn bench(c: &mut Criterion) {
    let mut game = Game::default();
    let mut outcome = None;
    c.bench_function("Engine against self", |b| {
        b.iter(|| {
            if let Some(m) = game.get_engine_move_minimax(2) {
                game.play(&m);
            } else {
                if outcome.is_none() {
                    outcome = Some(format!(
                        "Game ended in {:?}. {} nodes searched.",
                        game.position.state, game.nodes_seached
                    ));
                }

                // Reset the board if no moves to play
                game = Game::default();
            }
        })
    });

    println!("{:?}", outcome);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench
}
criterion_main!(benches);
