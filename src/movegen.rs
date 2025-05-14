use crate::{
    bitboard::{BitBoard, EMPTY},
    board::{Board, Color},
    square::Square,
};

#[derive(PartialEq)]
pub struct Move(Square, Square);

/// Capturing NOT yet generated
pub fn generate_psuedo_legal_pawn_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();

    let occupied;
    let initial;
    let color;
    if board.is_whites_turn {
        occupied = board.white_pawn_bitboard;
        initial = BitBoard::INITIAL_WHITE_PAWN;
        color = Color::White;
    } else {
        occupied = board.black_pawn_bitboard;
        initial = BitBoard::INITIAL_BLACK_PAWN;
        color = Color::Black;
    }

    for p in occupied {
        match p.forward(&color) {
            Some(once) => {
                if board.determine_piece(once).is_some() {
                    continue;
                }

                moves.push(Move(p, once));

                if BitBoard::from_square(p) & initial != EMPTY {
                    let twice = once.forward(&color).unwrap();
                    if board.determine_piece(twice).is_some() {
                        continue;
                    }

                    moves.push(Move(p, twice));
                }
            }
            None => continue,
        }
    }

    moves
}

// pub fn generate_psuedo_legal_moves(board: Board) -> Vec<Move> {
//
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn psuedo_legal_pawn_moves() {
        let mut board = Board::default();
        let moves = generate_psuedo_legal_pawn_moves(&board);

        // White pawn single push
        let expected_move = Move(Square::C2, Square::C3);
        assert!(
            moves.contains(&expected_move),
            "Valid white move deemed invalid."
        );

        // White pawn invalid long push
        let not_expected_move = Move(Square::G2, Square::G5);
        assert!(
            !moves.contains(&not_expected_move),
            "Invalid white move deemed valid."
        );

        board.is_whites_turn = false;
        let moves = generate_psuedo_legal_pawn_moves(&board);
        // Black pawn double push from starting rank
        let double_push = Move(Square::H7, Square::H5);
        assert!(
            moves.contains(&double_push),
            "Valid black double push deemed invalid."
        );

        // Black pawn single push
        let single_push = Move(Square::E7, Square::E6);
        assert!(
            moves.contains(&single_push),
            "Valid black single push deemed invalid."
        );

        // Black pawn invalid move (3-step)
        let invalid_black_move = Move(Square::A7, Square::A4);
        assert!(
            !moves.contains(&invalid_black_move),
            "Invalid black move deemed valid."
        );
    }
}
