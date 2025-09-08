use whalecrab::board::Board;

fn main() {
    let mut board = Board::default();

    for _ in 0..100 {
        board = board.make_engine_move();
    }

    println!("Final score: {}", board.grade_position());
    println!("Final fen: {}", board.to_fen());
}
