use std::fmt::{self, Display};

use crate::{
    bitboard::{BitBoard, EMPTY},
    board::{Board, Color},
    square::Square,
};

#[derive(PartialEq)]
pub struct Move(pub Square, pub Square);

impl Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(from {} to {})", self.0, self.1)
    }
}

impl Move {
    /// Clones the bitboard, makes the move (captures is needed), and returns the new board. Does
    /// not verify legality at all.
    pub fn make(&self, board: &Board) -> Board {
        let color = board.determine_color(self.0).unwrap_or_else(|| {
            panic!(
                "Coudn't determine piece color while trying to make move: {}",
                self
            )
        });

        let piece = board.determine_piece(self.0).unwrap_or_else(|| {
            panic!("Tried to make move {}, but there is no piece to move", self)
        });

        let initial = BitBoard::from_square(self.0);
        let target = BitBoard::from_square(self.1);
        let mut new = board.clone();

        // Move the piece
        new.set_occupied_bitboard(
            &piece,
            &color,
            target | (initial ^ board.get_occupied_bitboard(&piece, &color)),
        );

        // Capture any potential piece on the target square
        if let Some(enemy_piece) = board.determine_piece(self.1) {
            let enemy_color = color.opponent();
            new.set_occupied_bitboard(
                &enemy_piece,
                &enemy_color,
                target ^ board.get_occupied_bitboard(&enemy_piece, &enemy_color),
            );
        }

        new.turn = board.turn.opponent();
        new
    }
}

/// Generates all legal moves for a single pawn
/// Capturing NOT yet generated
pub fn generate_psuedo_legal_pawn_targets(board: &Board, sq: Square) -> Vec<Square> {
    let mut targets = Vec::new();

    let color = board
        .determine_color(sq)
        .expect("Couldn't determine piece color!");
    let enemy_color = &color.opponent();

    let initial = match color {
        Color::White => BitBoard::INITIAL_WHITE_PAWN,
        Color::Black => BitBoard::INITIAL_BLACK_PAWN,
    };

    // Advances
    if let Some(once) = sq.forward(&color) {
        if board.determine_piece(once).is_some() {
            return targets;
        }

        targets.push(once);

        // If on initial rank
        if sq.in_bitboard(&initial) {
            let twice = once.forward(&color).unwrap();
            if board.determine_piece(twice).is_some() {
                return targets;
            }

            targets.push(twice);
        }
    }

    // Captures
    if let Some(left) = sq.fleft(&color) {
        if let Some(target) = board.determine_color(left) {
            if target == *enemy_color {
                targets.push(left);
            }
        }
    }

    if let Some(right) = sq.fright(&color) {
        if let Some(target) = board.determine_color(right) {
            if target == *enemy_color {
                targets.push(right);
            }
        }
    }

    targets
}

/// Generates all moves for all pawn
pub fn generate_all_psuedo_legal_pawn_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();

    let occupied = match board.turn {
        Color::White => board.white_pawn_bitboard,
        Color::Black => board.black_pawn_bitboard,
    };

    for p in occupied {
        let targets = generate_psuedo_legal_pawn_targets(board, p);
        for sq in targets {
            moves.push(Move(p, sq));
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
        let moves = generate_all_psuedo_legal_pawn_moves(&board);

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

        board.turn = board.turn.opponent();
        let moves = generate_all_psuedo_legal_pawn_moves(&board);
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

    #[test]
    fn black_pawn_sees_white_target() {
        let mut board = Board::default();
        let looking_for = Move(Square::D5, Square::C4);
        for m in [
            &Move(Square::C2, Square::C4),
            &Move(Square::D7, Square::D5),
            &Move(Square::H2, Square::H3),
        ] {
            board = m.make(&board);
        }

        assert_eq!(board.turn, Color::Black);
        assert!(
            looking_for.1.in_bitboard(&board.white_pawn_bitboard),
            "White pawn not in position"
        );
        assert!(
            looking_for.0.in_bitboard(&board.black_pawn_bitboard),
            "Black pawn not in position"
        );
        let moves = generate_psuedo_legal_pawn_targets(&board, looking_for.0);
        assert!(
            moves.contains(&looking_for.1),
            "Black pawn can't see target. Availabe moves: {:?}",
            moves
        );
    }

    #[test]
    fn white_pawn_captures_black_pawn() {
        let mut board = Board::default();
        let capture = Move(Square::B4, Square::C5);
        let white_pawns_before = board.white_pawn_bitboard.popcnt();
        let black_pawns_before = board.black_pawn_bitboard.popcnt();

        for m in [
            &Move(Square::B2, Square::B4),
            &Move(Square::C7, Square::C5),
            &capture,
        ] {
            board = m.make(&board);
        }

        let white_pawns_after = board.white_pawn_bitboard.popcnt();
        let black_pawns_after = board.black_pawn_bitboard.popcnt();

        assert!(
            !Square::B2.in_bitboard(&board.white_pawn_bitboard),
            "White never moved"
        );
        assert!(
            !capture.0.in_bitboard(&board.white_pawn_bitboard),
            "White moved but failed to capture"
        );
        assert!(
            !capture.1.in_bitboard(&board.black_pawn_bitboard),
            "The black pawn is still standing"
        );
        assert!(
            capture.1.in_bitboard(&board.white_pawn_bitboard),
            "White isn't in the correct posistion"
        );

        assert_ne!(
            black_pawns_before, black_pawns_after,
            "The number of black pawns didn't change"
        );
        assert_eq!(
            black_pawns_before - 1,
            black_pawns_after,
            "The number of black pawns didn't decrease by one"
        );
        assert_eq!(
            white_pawns_before, white_pawns_after,
            "The number of white pawns changed"
        );
    }

    #[test]
    fn make_moves() {
        let original = Board::default();

        let pawn = Move(Square::C2, Square::C3);
        let knight = Move(Square::G8, Square::F6);
        let king = Move(Square::E1, Square::E2);

        let after_pawn = pawn.make(&original);
        let after_knight = knight.make(&original);
        let after_king = king.make(&original);

        assert!(pawn.0.in_bitboard(&original.white_pawn_bitboard));
        assert!(!pawn.1.in_bitboard(&original.white_pawn_bitboard));
        assert!(!pawn.0.in_bitboard(&after_pawn.white_pawn_bitboard));
        assert!(pawn.1.in_bitboard(&after_pawn.white_pawn_bitboard));

        assert!(knight.0.in_bitboard(&original.black_knight_bitboard));
        assert!(!knight.1.in_bitboard(&original.black_knight_bitboard));
        assert!(!knight.0.in_bitboard(&after_knight.black_knight_bitboard));
        assert!(knight.1.in_bitboard(&after_knight.black_knight_bitboard));

        assert!(king.0.in_bitboard(&original.white_king_bitboard));
        assert!(!king.1.in_bitboard(&original.white_king_bitboard));
        assert!(!king.0.in_bitboard(&after_king.white_king_bitboard));
        assert!(king.1.in_bitboard(&after_king.white_king_bitboard));
    }
}
