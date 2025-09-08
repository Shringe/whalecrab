use rand::Rng;
use whalecrab::board::{Board, Color};

fn main() {
    let mut board = Board::default();
    let mut rng = rand::rng();

    for _ in 0..100 {
        match board.turn {
            Color::White => board = board.make_engine_move(),
            Color::Black => {
                let moves = board.generate_all_legal_moves();
                let chosen_move = rng.random_range(0..moves.len());
                let m = moves.get(chosen_move).expect("Chose invalid move");
                board = m.make(&board);
            }
        }
    }

    println!("Final score: {}", board.grade_position());
    println!("Final fen: {}", board.to_fen());
}
