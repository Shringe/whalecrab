use rand::Rng;
use whalecrab_engine::{engine::Engine, timers::infinite::Infinite};
use whalecrab_lib::movegen::pieces::piece::PieceColor;

fn main() {
    let mut engine = Engine::default();
    let mut rng = rand::rng();

    for _ in 0..100 {
        let m = match engine.game.turn {
            PieceColor::White => engine.minimax(&Infinite, 2).best_move,
            PieceColor::Black => {
                let moves = engine.game.legal_moves();
                if moves.is_empty() {
                    None
                } else {
                    let chosen_move = rng.random_range(0..moves.len());
                    let m = moves.get(chosen_move).expect("Chose invalid move");
                    Some(*m)
                }
            }
        };

        match m {
            Some(m) => {
                println!("Chose to play: {}", m);
                engine.game.play(&m);
            }
            None => {
                println!("Game ended in {:?}.", engine.game.state);
                break;
            }
        }
    }

    println!("=========================");
    println!("Final score: {}", engine.grade_position());
    println!("Final fen: {}", engine.game.to_fen());
}
