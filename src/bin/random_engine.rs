use rand::Rng;
use whalecrab::board::Board;

fn main() {
    let mut board = Board::default();
    let mut rng = rand::rng();

    for _ in 0..10 {
        let moves = board.generate_all_legal_moves();
        let chosen_move = rng.random_range(0..moves.len());
        let m = moves.get(chosen_move).expect("Chose invalid move");
        println!("Chose to play: {}", m);
        board = m.make(&board);
    }

    println!("Final fen: {}", board.to_fen());
}
