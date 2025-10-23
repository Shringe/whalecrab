use whalecrab_lib::{board::State, game::Game, movegen::pieces::piece::Color};

fn main() {
    let mut game = Game::default();

    while game.position.state == State::InProgress {
        let m = match game.position.turn {
            Color::White => game.get_engine_move_minimax(3),
            Color::Black => game.get_engine_move_minimax(2),
        };

        match m {
            Some(m) => {
                game.play(&m);
                println!(
                    "Game score {} after chosing to play: {}, black_pawns: {}, white_pawns: {}\n  fen: '{}'",
                    game.grade_position(),
                    m,
                    game.position.black_pawns.popcnt(),
                    game.position.white_pawns.popcnt(),
                    game.position.to_fen(),
                );
            }
            None => {
                println!("Game ended in {:?}.", game.position.state);
                break;
            }
        }
    }

    println!("=========================");
    println!("Nodes searched: {}", game.nodes_seached);
    println!("Final state: {:?}", game.position.state);
    println!("Final score: {}", game.grade_position());
    println!("Final fen: {}", game.position.to_fen());
}
