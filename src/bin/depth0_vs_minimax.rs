use whalecrab::board::{Board, Color};

fn main() {
    let mut board = Board::default();

    for _ in 0..40 {
        let m = match board.turn {
            Color::White => board.find_best_move().unwrap().0,
            Color::Black => board.get_engine_move_minimax(3).unwrap(),
        };

        println!("{:?}, Chose to play: {}", board.turn, m);
        board = m.make(&board);
    }

    println!("=========================");
    println!("Final score: {}", board.grade_position());
    println!("Final fen: {}", board.to_fen());
}
