use whalecrab_engine::{engine::Engine, timers::infinite::Infinite};
use whalecrab_lib::{movegen::pieces::piece::PieceColor, position::game::State};

fn main() {
    let mut engine = Engine::default();

    while engine.game.state == State::InProgress {
        let m = match engine.game.turn {
            PieceColor::White => engine.minimax(&Infinite, 3),
            PieceColor::Black => engine.minimax(&Infinite, 2),
        }
        .best_move;

        match m {
            Some(m) => {
                engine.game.play(&m);
                println!(
                    "Game score {} after chosing to play: {}, black_pawns: {}, white_pawns: {}\n  fen: '{}'",
                    engine.grade_position(),
                    m,
                    engine.game.black_pawns.popcnt(),
                    engine.game.white_pawns.popcnt(),
                    engine.game.to_fen(),
                );
            }
            None => {
                println!("Game ended in {:?}.", engine.game.state);
                break;
            }
        }
    }

    println!("=========================");
    println!("Nodes searched: {}", engine.nodes_searched);
    println!("Final state: {:?}", engine.game.state);
    println!("Final score: {}", engine.grade_position());
    println!("Final fen: {}", engine.game.to_fen());
}
