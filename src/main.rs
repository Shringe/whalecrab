pub mod board;
mod generator;

use board::render_bitboard;
use board::Board;

fn main() {
    let board = Board::new();

    // println!("{:?}", board);
    println!("{}", render_bitboard(board.black_pawn_bitboard));
}
