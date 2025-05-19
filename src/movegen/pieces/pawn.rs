use crate::{
    bitboard::BitBoard,
    board::{Board, Color},
    movegen::moves::{Move, SpecialMove},
    square::Square,
};

use super::piece::Piece;

pub struct Pawn(pub Square);

impl Piece for Pawn {
    /// Generates all psuedo legal moves for a single pawn
    /// En_Passant is considered
    /// Promotion not considered
    /// King safety not considered
    fn psuedo_legal_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let color = board
            .determine_color(self.0)
            .expect("Couldn't determine piece color!");
        let enemy_color = &color.opponent();

        let initial = match color {
            Color::White => BitBoard::INITIAL_WHITE_PAWN,
            Color::Black => BitBoard::INITIAL_BLACK_PAWN,
        };

        // Advances
        if let Some(once) = self.0.forward(&color) {
            if board.determine_piece(once).is_none() {
                moves.push(Move {
                    from: self.0,
                    to: once,
                    special: None,
                });
            }

            // If on initial rank
            if self.0.in_bitboard(&initial) {
                let twice = once.forward(&color).unwrap();
                if board.determine_piece(twice).is_none() {
                    moves.push(Move {
                        from: self.0,
                        to: twice,
                        special: Some(SpecialMove::CreateEnPassant),
                    });
                }
            }
        }

        // Captures
        for diagnol in [self.0.fleft(&color), self.0.fright(&color)]
            .into_iter()
            .flatten()
        {
            if let Some(enemy) = board.determine_color(diagnol) {
                if enemy == *enemy_color {
                    moves.push(Move {
                        from: self.0,
                        to: diagnol,
                        special: None,
                    });
                }
            } else if let Some(target) = board.en_passant_target {
                if diagnol == target {
                    moves.push(Move {
                        from: self.0,
                        to: target,
                        special: Some(SpecialMove::CaptureEnPassant),
                    });
                }
            }
        }

        moves
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::format_pretty_list;

    use super::*;

    #[test]
    fn white_pawn_sees_black_target() {
        let mut board = Board::default();
        let looking_for = Move {
            from: Square::H4,
            to: Square::G5,
            special: None,
        };
        for m in [
            Move {
                from: Square::H2,
                to: Square::H4,
                special: None,
            },
            Move {
                from: Square::G7,
                to: Square::G5,
                special: None,
            },
        ] {
            board = m.make(&board);
        }

        assert_eq!(board.turn, Color::White);
        assert!(
            looking_for.to.in_bitboard(&board.black_pawn_bitboard),
            "Black pawn not in position"
        );
        assert!(
            looking_for.from.in_bitboard(&board.white_pawn_bitboard),
            "White pawn not in position"
        );
        let moves = Pawn(looking_for.from).psuedo_legal_moves(&board);
        assert!(
            moves.contains(&looking_for),
            "White pawn can't see target. {}",
            format_pretty_list(&moves)
        );
    }

    #[test]
    fn black_pawn_sees_white_target() {
        let mut board = Board::default();
        let looking_for = Move {
            from: Square::D5,
            to: Square::C4,
            special: None,
        };
        for m in [
            Move {
                from: Square::C2,
                to: Square::C4,
                special: Some(SpecialMove::CreateEnPassant),
            },
            Move {
                from: Square::D7,
                to: Square::D5,
                special: None,
            },
            Move {
                from: Square::H2,
                to: Square::H3,
                special: None,
            },
        ] {
            board = m.make(&board);
        }

        assert_eq!(board.turn, Color::Black);
        assert!(
            looking_for.to.in_bitboard(&board.white_pawn_bitboard),
            "White pawn not in position"
        );
        assert!(
            looking_for.from.in_bitboard(&board.black_pawn_bitboard),
            "Black pawn not in position"
        );
        let moves = Pawn(looking_for.from).psuedo_legal_moves(&board);
        assert!(
            moves.contains(&looking_for),
            "Black pawn can't see target. Available moves: {:?}",
            moves
        );
    }
}
