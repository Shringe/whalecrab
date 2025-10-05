use whalecrab_lib::{board::Color, game::Game};

fn main() {
    let mut game = Game::default();

    for _ in 0..40 {
        let m = match game.position.turn {
            Color::White => {
                if let Some(sm) = game.find_best_move() {
                    sm.0
                } else {
                    println!("White has no more moves!");
                    break;
                }
            }
            Color::Black => {
                if let Some(m) = game.get_engine_move_minimax(3) {
                    m
                } else {
                    println!("Black has no more moves!");
                    break;
                }
            }
        };

        println!("{:?}, Chose to play: {}", game.position.turn, m);
        game.play(&m);
    }

    println!("=========================");
    println!("Final score: {}", game.grade_position());
    println!("Final fen: {}", game.position.to_fen());
}
