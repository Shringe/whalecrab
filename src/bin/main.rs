use crabfish::board::Board;
use crabfish::pieces::white_pawn::WhitePawn;
use crabfish::square::Square;

fn main() {
    let board = Board::new();

    let white_pawn = WhitePawn {
        // position: 2,
        // position: 0b00000000_00100000_00000000_00000000_00000000_00000000_00000000_00000000,
        position: Square::E2,
        board,
    };

    // println!("{:?}", board);
    println!("Is here:\n{:?}", white_pawn.position);

    // println!(
    //     "\nCan move:\n{}",
    //     render_bitboard(white_pawn.psuedo_legal_moves())
    // );
}
