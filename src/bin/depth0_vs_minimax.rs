use whalecrab::board::{Board, Color};

fn main() {
    let mut board = Board::default();

    for _ in 0..40 {
        let m = match board.turn {
            Color::White => {
                if let Some(sm) = board.find_best_move() {
                    sm.0
                } else {
                    println!("White has no more moves!");
                    break;
                }
            }
            Color::Black => {
                if let Some(m) = board.get_engine_move_minimax(3) {
                    m
                } else {
                    println!("Black has no more moves!");
                    break;
                }
            }
        };

        println!("{:?}, Chose to play: {}", board.turn, m);
        board = m.make(&board);
    }

    println!("=========================");
    println!("Final score: {}", board.grade_position());
    println!("Final fen: {}", board.to_fen());
}
