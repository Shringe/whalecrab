use whalecrab_engine::engine::Engine;
use whalecrab_lib::{game::State, movegen::pieces::piece::PieceColor};

fn main() {
    let mut engine = Engine::default();

    while engine.game.state == State::InProgress {
        let m = match engine.game.turn {
            PieceColor::White => engine.minimax(3),
            PieceColor::Black => engine.minimax(2),
        };

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
