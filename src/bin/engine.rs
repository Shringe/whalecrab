use whalecrab::board::Board;

fn main() {
    let mut board = Board::default();

    for _ in 0..10 {
        board = board.make_engine_move();
    }

    println!("Final fen: {}", board.to_fen());
}
