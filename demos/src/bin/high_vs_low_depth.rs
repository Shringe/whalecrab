use whalecrab_lib::{game::Game, movegen::pieces::piece::Color};

fn main() {
    let mut game = Game::default();

    for _ in 0..100 {
        let m = match game.position.turn {
            Color::White => game.get_engine_move_minimax(2),
            Color::Black => game.get_engine_move_minimax(2),
        };

        match m {
            Some(m) => {
                println!("Chose to play: {}", m);
                game.play(&m);
            }
            None => {
                println!("Game ended in {:?}.", game.position.state);
                break;
            }
        }
    }

    println!("=========================");
    println!("Final score: {}", game.grade_position());
    println!("Final fen: {}", game.position.to_fen());
}
