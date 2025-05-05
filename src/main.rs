pub mod board;
pub mod pieces;

use board::render_bitboard;
use board::Board;
use pieces::white_pawn;

fn main() {
    let board = Board::new();

    let white_pawn = pieces::white_pawn::WhitePawn {
        // position: 2,
        position: 0b00000000_00100000_00000000_00000000_00000000_00000000_00000000_00000000,
        board,
    };

    // println!("{:?}", board);
    println!("Is here:\n{}", render_bitboard(white_pawn.position));
    println!(
        "\nCan move:\n{}",
        render_bitboard(white_pawn.psuedo_legal_moves())
    );
}
