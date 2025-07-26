use std::fmt;

use crate::{
    bitboard::BitBoard, board::{Board, Color, PieceType}, castling::{self, CastleSide}, square::Square
};

/// Provides information of what to remove from the game after a piece gets captured
pub struct Capture(PieceType, Square);

#[derive(PartialEq, Debug)]
pub enum MoveType {
    Normal, // Includes regular captures
    CreateEnPassant,
    CaptureEnPassant,
    Promotion(PieceType),
    Castle(CastleSide),
}

#[derive(PartialEq, Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub variant: MoveType,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}, {:?}", self.from, self.to, self.variant)
    }
}

impl Move {
    /// Infers the type of variant move. This is likely already known during move generation, and in that
    /// case it is recommended to skip using this contructor.
    pub fn new(from: Square, to: Square, board: &Board) -> Self {
        Self {
            from,
            to,
            variant: match (&board.turn, from, to) {
                (Color::White, Square::E1, Square::C1) if board.castling_rights.white_queenside => MoveType::Castle(CastleSide::Queenside),
                (Color::White, Square::E1, Square::G1) if board.castling_rights.white_kingside => MoveType::Castle(CastleSide::Kingside),
                (Color::Black, Square::E8, Square::C8) if board.castling_rights.black_queenside => MoveType::Castle(CastleSide::Queenside),
                (Color::Black, Square::E8, Square::G8) if board.castling_rights.black_kingside => MoveType::Castle(CastleSide::Kingside),
                _ => {
                    if let Some(target) = board.en_passant_target {
                        if to == target && board.determine_piece(from) == Some(PieceType::Pawn) {
                            MoveType::CaptureEnPassant
                        } else {
                            MoveType::Normal
                        }
                    } else if board.determine_piece(from) == Some(PieceType::Pawn) {
                        let color = board.determine_color(from).unwrap();
                        if let Some(once) = from.forward(&color) {
                            if let Some(twice) = once.forward(&color) {
                                if to == twice {
                                    MoveType::CreateEnPassant
                                } else {
                                    MoveType::Normal
                                }
                            } else if once.get_rank() == color.final_rank() {
                                MoveType::Promotion(PieceType::Queen)
                            } else {
                                MoveType::Normal
                            }
                        } else {
                            MoveType::Normal
                        }
                    } else {
                        MoveType::Normal
                    }
                }
            }
        }
    }

    /// Clones the bitboard, makes the move (captures if needed), and returns the new board. Does
    /// not verify legality at all.
    pub fn make(&self, board: &Board) -> Board {
        let mut new = board.clone();
        let color = board.determine_color(self.from).unwrap_or_else(|| {
            panic!(
                "Coudn't determine piece color while trying to make move: {}",
                self
            )
        });

        match &self.variant {
            MoveType::Castle(side) => {
                match &color {
                    Color::White => {
                        new.castling_rights.white_queenside = false;
                        new.castling_rights.white_kingside = false;

                        match side {
                            CastleSide::Queenside => {
                                new.white_king_bitboard = new.white_king_bitboard ^ castling::WHITE_CASTLE_QUEENSIDE_KING_MOVES;
                                new.white_rook_bitboard = new.white_rook_bitboard ^ castling::WHITE_CASTLE_QUEENSIDE_ROOK_MOVES;
                            },
                            CastleSide::Kingside => {
                                new.white_king_bitboard = new.white_king_bitboard ^ castling::WHITE_CASTLE_KINGSIDE_KING_MOVES;
                                new.white_rook_bitboard = new.white_rook_bitboard ^ castling::WHITE_CASTLE_KINGSIDE_ROOK_MOVES;
                            },
                        }
                    },

                    Color::Black => {
                        new.castling_rights.black_queenside = false;
                        new.castling_rights.black_kingside = false;

                        match side {
                            CastleSide::Queenside => {
                                new.black_king_bitboard = new.black_king_bitboard ^ castling::BLACK_CASTLE_QUEENSIDE_KING_MOVES;
                                new.black_rook_bitboard = new.black_rook_bitboard ^ castling::BLACK_CASTLE_QUEENSIDE_ROOK_MOVES;
                            },

                            CastleSide::Kingside => {
                                new.black_king_bitboard = new.black_king_bitboard ^ castling::BLACK_CASTLE_KINGSIDE_KING_MOVES;
                                new.black_rook_bitboard = new.black_rook_bitboard ^ castling::BLACK_CASTLE_KINGSIDE_ROOK_MOVES;
                            },
                        }
                    },
                }

                new.next_turn();
                return new;
            },

            _ => {},
        }

        let (initial_piece, target_piece) = match &self.variant {
            MoveType::Promotion(promotion_piece) => (PieceType::Pawn, promotion_piece.clone()),
            _ => {
                let piece = board.determine_piece(self.from).unwrap_or_else(|| {
                    panic!("Tried to make move {}, but there is no piece to move", self)
                });

                (piece.clone(), piece.clone())
            }
        };

        let initial = BitBoard::from_square(self.from);
        let target = BitBoard::from_square(self.to);

        // Remove the piece from its original square
        new.set_occupied_bitboard(
            &initial_piece,
            &color,
            new.get_occupied_bitboard(&initial_piece, &color) ^ initial,
        );

        // Add the (possibly different) piece to the target square
        new.set_occupied_bitboard(
            &target_piece,
            &color,
            new.get_occupied_bitboard(&target_piece, &color) | target,
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
        if self.variant == MoveType::CreateEnPassant {
            new.en_passant_target = self.to.backward(&color);
        } 

        new
    }

    /// Gets the square and piece type of the captured piece
    fn get_capture(&self, board: &Board) -> Option<Capture> {
        let target = if self.variant == MoveType::CaptureEnPassant {
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
    use crate::castling::{BLACK_CASTLES_KINGSIDE, WHITE_CASTLES_QUEENSIDE};
    use crate::movegen::pieces::king::King;
    use crate::movegen::pieces::pawn::Pawn;
    use crate::movegen::pieces::piece::Piece;
    use crate::test_utils::{compare_to_fen, format_pretty_list, should_generate};

    #[test]
    fn white_king_castles_queenside() {
        let fen_before = "rn2k2r/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/R3K1NR w KQkq - 6 7";
        let fen_after = "rn2k2r/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/2KR2NR b kq - 7 7";
        let to_play = &WHITE_CASTLES_QUEENSIDE;
        let mut board = Board::from_fen(fen_before).unwrap();

        let moves = King(to_play.from).psuedo_legal_moves(&board);
        should_generate(&moves, to_play);

        board = to_play.make(&board);
        compare_to_fen(&board, fen_after);
    }

    #[test]
    fn black_king_castles_kingside() {
        let fen_before = "rn2k2r/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/2KR2NR b kq - 7 7";
        let fen_after = "rn3rk1/pppbqppp/3p1n2/2b1p3/2B1P3/2NP4/PPPBQPPP/2KR2NR w - - 8 8";
        let to_play = &BLACK_CASTLES_KINGSIDE;
        let mut board = Board::from_fen(fen_before).unwrap();

        let moves = King(to_play.from).psuedo_legal_moves(&board);
        should_generate(&moves, to_play);

        board = to_play.make(&board);
        compare_to_fen(&board, fen_after);
    }

    #[test]
    fn white_pawn_promotes_to_queen() {
        let mut board = Board::default();
        let looking_for = Move {
            from: Square::G7,
            to: Square::H8,
            variant: MoveType::Promotion(PieceType::Queen),
        };

        for m in [
            Move {
                from: Square::H2,
                to: Square::H4,
                variant: MoveType::CreateEnPassant,
            },
            Move {
                from: Square::G7,
                to: Square::G5,
                variant: MoveType::CreateEnPassant,
            },
            Move {
                from: Square::H4,
                to: Square::G5,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::H7,
                to: Square::H6,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::G5,
                to: Square::H6,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::F8,
                to: Square::G7,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::H6,
                to: Square::G7,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::E7,
                to: Square::E5,
                variant: MoveType::CreateEnPassant,
            },
        ] {
            board = m.make(&board);
        }

        assert_eq!(board.turn, Color::White);
        assert!(
            looking_for.from.in_bitboard(&board.white_pawn_bitboard),
            "White pawn not in position"
        );
        assert!(
            looking_for.to.in_bitboard(&board.black_rook_bitboard),
            "Black rook not in position"
        );
        let moves = Pawn(looking_for.from).psuedo_legal_moves(&board);
        assert!(
            moves.contains(&looking_for),
            "White pawn can't see target. Available moves: {:?}",
            moves
        );

        let after = looking_for.make(&board);

        assert_eq!(after.turn, Color::Black);
        assert!(
            Square::H8.in_bitboard(&after.white_queen_bitboard),
            "Expected white queen at H8 after promotion"
        );
        assert!(
            !Square::H8.in_bitboard(&after.white_pawn_bitboard),
            "H8 incorrectly contains a white pawn after promotion"
        );
        assert!(
            !looking_for.from.in_bitboard(&after.white_pawn_bitboard),
            "Original white pawn still present at {} after promotion",
            looking_for.from
        );
    }

    #[test]
    fn make_moves() {
        let original = Board::default();

        let pawn = Move {
            from: Square::C2,
            to: Square::C3,
            variant: MoveType::Normal,
        };
        let knight = Move {
            from: Square::G8,
            to: Square::F6,
            variant: MoveType::Normal,
        };
        let king = Move {
            from: Square::E1,
            to: Square::E2,
            variant: MoveType::Normal,
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
            variant: MoveType::Normal,
        };
        let white_pawns_before = board.white_pawn_bitboard.popcnt();
        let black_pawns_before = board.black_pawn_bitboard.popcnt();

        for m in [
            &Move {
                from: Square::B2,
                to: Square::B4,
                variant: MoveType::Normal,
            },
            &Move {
                from: Square::C7,
                to: Square::C5,
                variant: MoveType::Normal,
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
            variant: MoveType::CaptureEnPassant,
        };
        for m in [
            Move {
                from: Square::D2,
                to: Square::D3,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::B7,
                to: Square::B5,
                variant: MoveType::CreateEnPassant,
            },
            Move {
                from: Square::D3,
                to: Square::D4,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::B5,
                to: Square::B4,
                variant: MoveType::Normal,
            },
            Move {
                from: Square::C2,
                to: Square::C4,
                variant: MoveType::CreateEnPassant,
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
