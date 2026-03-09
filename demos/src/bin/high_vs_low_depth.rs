use whalecrab_engine::engine::Engine;
use whalecrab_lib::{board::State, movegen::pieces::piece::PieceColor};

fn main() {
    let mut engine = Engine::default();

    while engine.game.position.state == State::InProgress {
        let m = match engine.game.position.turn {
            PieceColor::White => engine.get_engine_move_minimax(3),
            PieceColor::Black => engine.get_engine_move_minimax(2),
        };

        match m {
            Some(m) => {
                engine.game.play(&m);
                println!(
                    "Game score {} after chosing to play: {}, black_pawns: {}, white_pawns: {}\n  fen: '{}'",
                    engine.grade_position(),
                    m,
                    engine.game.position.black_pawns.popcnt(),
                    engine.game.position.white_pawns.popcnt(),
                    engine.game.position.to_fen(),
                );
            }
            None => {
                println!("Game ended in {:?}.", engine.game.position.state);
                break;
            }
        }
    }

    println!("=========================");
    println!("Nodes searched: {}", engine.game.nodes_seached);
    println!("Final state: {:?}", engine.game.position.state);
    println!("Final score: {}", engine.grade_position());
    println!("Final fen: {}", engine.game.position.to_fen());
}
