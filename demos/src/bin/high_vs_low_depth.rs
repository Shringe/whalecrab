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
                game.play(&m);
                println!(
                    "Game score {} after chosing to play: {}",
                    game.grade_position(),
                    m
                );
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
