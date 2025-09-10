use whalecrab::board::Board;

fn main() {
    let mut board = Board::default();

    for _ in 0..100 {
        let m = board.find_best_move().unwrap().0;
        println!("Chose to play: {}", m);
        board = m.make(&board);
    }

    println!("=========================");
    println!("Final score: {}", board.grade_position());
    println!("Final fen: {}", board.to_fen());
}
