use criterion::Criterion;
use whalecrab_engine::engine::Engine;
use whalecrab_lib::game::Game;
mod common;

fn bench(c: &mut Criterion) {
    let mut engine = Engine::default();
    let mut outcome = None;
    c.bench_function("Engine against self", |b| {
        b.iter(|| {
            if let Some(m) = engine.get_engine_move_minimax(2) {
                engine.game.play(&m);
            } else {
                if outcome.is_none() {
                    outcome = Some(format!(
                        "Game ended in {:?}. {} nodes searched.",
                        engine.game.position.state, engine.game.nodes_seached
                    ));
                }

                // Reset the board if no moves to play
                engine.with_new_game(Game::default());
            }
        })
    });

    println!("{:?}", outcome);
}

setup_criterion!();
