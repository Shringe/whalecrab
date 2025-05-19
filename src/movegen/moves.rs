use std::fmt;

use crate::{
    bitboard::BitBoard,
    board::{Board, PieceType},
    square::Square,
};

/// Provides information of what to remove from the game after a piece gets captured
pub struct Capture(PieceType, Square);

#[derive(PartialEq, Debug)]
pub enum SpecialMove {
    CreateEnPassant,
    CaptureEnPassant,
}

#[derive(PartialEq, Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,

    pub special: Option<SpecialMove>,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}, {:?}", self.from, self.to, self.special)
    }
}

impl Move {
    /// Infers the type of special move. This is likely already known during move generation, and in that
    /// case it is recommended to skip using this contructor.
    pub fn new(from: Square, to: Square, board: &Board) -> Self {
        Self {
            from,
            to,
            special: if let Some(target) = board.en_passant_target {
                if to == target && board.determine_piece(from) == Some(PieceType::Pawn) {
                    Some(SpecialMove::CaptureEnPassant)
                } else {
                    None
                }
            } else if board.determine_piece(from) == Some(PieceType::Pawn) {
                let color = board.determine_color(from).unwrap();
                if let Some(once) = from.forward(&color) {
                    if let Some(twice) = once.forward(&color) {
                        if to == twice {
                            Some(SpecialMove::CreateEnPassant)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            },
        }
    }

    /// Clones the bitboard, makes the move (captures if needed), and returns the new board. Does
    /// not verify legality at all.
    pub fn make(&self, board: &Board) -> Board {
        let color = board.determine_color(self.from).unwrap_or_else(|| {
            panic!(
                "Coudn't determine piece color while trying to make move: {}",
                self
            )
        });

        let piece = board.determine_piece(self.from).unwrap_or_else(|| {
            panic!("Tried to make move {}, but there is no piece to move", self)
        });

        let initial = BitBoard::from_square(self.from);
        let target = BitBoard::from_square(self.to);
        let mut new = board.clone();

        // Move the piece
        new.set_occupied_bitboard(
            &piece,
            &color,
            target | (initial ^ board.get_occupied_bitboard(&piece, &color)),
        );

        // Capture any potential piece on the target square
        if let Some(capture) = self.get_capture(board) {
            let enemy_color = color.opponent();
            new.set_occupied_bitboard(
                &capture.0,
                &enemy_color,
                BitBoard::from_square(capture.1)
                    ^ board.get_occupied_bitboard(&capture.0, &enemy_color),
            );
        }

        // Set en passant rules and switch turn
        new.next_turn();
        if let Some(special) = &self.special {
            if *special == SpecialMove::CreateEnPassant {
                new.en_passant_target = self.to.backward(&color);
            }
        }

        new
    }

    /// Gets the square and piece type of the captured piece
    fn get_capture(&self, board: &Board) -> Option<Capture> {
        let target = if self.special == Some(SpecialMove::CaptureEnPassant) {
            self.to
                .backward(&board.turn)
                .expect("Invalid en passant square")
        } else {
            self.to
        };

        board
            .determine_piece(target)
            .map(|piece| Capture(piece, target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Color;
    use crate::movegen::pieces::pawn::Pawn;
    use crate::movegen::pieces::piece::Piece;
    use crate::test_utils::*;

    #[test]
    fn make_moves() {
        let original = Board::default();

        let pawn = Move {
            from: Square::C2,
            to: Square::C3,
            special: None,
        };
        let knight = Move {
            from: Square::G8,
            to: Square::F6,
            special: None,
        };
        let king = Move {
            from: Square::E1,
            to: Square::E2,
            special: None,
        };

        let after_pawn = pawn.make(&original);
        let after_knight = knight.make(&original);
        let after_king = king.make(&original);

        assert!(pawn.from.in_bitboard(&original.white_pawn_bitboard));
        assert!(!pawn.to.in_bitboard(&original.white_pawn_bitboard));
        assert!(!pawn.from.in_bitboard(&after_pawn.white_pawn_bitboard));
        assert!(pawn.to.in_bitboard(&after_pawn.white_pawn_bitboard));

        assert!(knight.from.in_bitboard(&original.black_knight_bitboard));
        assert!(!knight.to.in_bitboard(&original.black_knight_bitboard));
        assert!(!knight.from.in_bitboard(&after_knight.black_knight_bitboard));
        assert!(knight.to.in_bitboard(&after_knight.black_knight_bitboard));

        assert!(king.from.in_bitboard(&original.white_king_bitboard));
        assert!(!king.to.in_bitboard(&original.white_king_bitboard));
        assert!(!king.from.in_bitboard(&after_king.white_king_bitboard));
        assert!(king.to.in_bitboard(&after_king.white_king_bitboard));
    }

    #[test]
    fn white_pawn_captures_black_pawn() {
        let mut board = Board::default();
        let capture = Move {
            from: Square::B4,
            to: Square::C5,
            special: None,
        };
        let white_pawns_before = board.white_pawn_bitboard.popcnt();
        let black_pawns_before = board.black_pawn_bitboard.popcnt();

        for m in [
            &Move {
                from: Square::B2,
                to: Square::B4,
                special: None,
            },
            &Move {
                from: Square::C7,
                to: Square::C5,
                special: None,
            },
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
            !capture.from.in_bitboard(&board.white_pawn_bitboard),
            "White moved but failed to capture"
        );
        assert!(
            !capture.to.in_bitboard(&board.black_pawn_bitboard),
            "The black pawn is still standing"
        );
        assert!(
            capture.to.in_bitboard(&board.white_pawn_bitboard),
            "White isn't in the correct position"
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
    fn black_pawn_takes_en_passant_target() {
        let mut board = Board::default();
        let capture = Move {
            from: Square::B4,
            to: Square::C3,
            special: Some(SpecialMove::CaptureEnPassant),
        };
        for m in [
            Move {
                from: Square::D2,
                to: Square::D3,
                special: None,
            },
            Move {
                from: Square::B7,
                to: Square::B5,
                special: Some(SpecialMove::CreateEnPassant),
            },
            Move {
                from: Square::D3,
                to: Square::D4,
                special: None,
            },
            Move {
                from: Square::B5,
                to: Square::B4,
                special: None,
            },
            Move {
                from: Square::C2,
                to: Square::C4,
                special: Some(SpecialMove::CreateEnPassant),
            },
        ] {
            board = m.make(&board);
        }

        assert_eq!(board.turn, Color::Black);
        assert!(
            capture.from.in_bitboard(&board.black_pawn_bitboard),
            "Black pawn not in position"
        );

        let moves = Pawn(capture.from).psuedo_legal_moves(&board);
        assert!(
            moves.contains(&capture),
            "Black pawn doesn't see en passant target. {}",
            format_pretty_list(&moves)
        );

        let white_pawns_before = board.white_pawn_bitboard.popcnt();
        let black_pawns_before = board.black_pawn_bitboard.popcnt();
        board = capture.make(&board);
        let white_pawns_after = board.white_pawn_bitboard.popcnt();
        let black_pawns_after = board.black_pawn_bitboard.popcnt();

        assert_eq!(black_pawns_before, black_pawns_after);
        assert_eq!(
            white_pawns_before - 1,
            white_pawns_after,
            "The white target is still standing"
        );
    }
}
