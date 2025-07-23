use crate::{
    bitboard::EMPTY, board::{Board, Color}, castling::{BLACK_TRAVERSES_CASTLING_KINGSIDE, BLACK_TRAVERSES_CASTLING_QUEENSIDE, WHITE_TRAVERSES_CASTLING_KINGSIDE, WHITE_TRAVERSES_CASTLING_QUEENSIDE}, movegen::moves::{Move, MoveType}, square::{Square, ALL_DIRECTIONS}
};

use super::piece::Piece;

pub struct King(pub Square);

impl Piece for King {
    /// King safety not considered. Castling not yet implemented.
    fn psuedo_legal_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();
        let enemy = board.turn.opponent();

        for d in ALL_DIRECTIONS {
            if let Some(sq) = self.0.walk(&d) {
                if let Some(piece) = board.determine_color(sq) {
                    if piece == enemy {
                        moves.push(Move {
                            from: self.0,
                            to: sq,
                            variant: MoveType::Normal,
                        })
                    }
                } else {
                    moves.push(Move {
                        from: self.0,
                        to: sq,
                        variant: MoveType::Normal,
                    })
                }
            }
        }

        let occupied = board.occupied_bitboard();
        match board.turn {
            Color::White => {
                if board.castling_rights.white_queenside && occupied & WHITE_TRAVERSES_CASTLING_QUEENSIDE == EMPTY {
                    moves.push(Move {
                        from: Square::E1,
                        to: Square::C1,
                        variant: MoveType::CastleQueenside,
                    })
                }

                if board.castling_rights.white_kingside && occupied & WHITE_TRAVERSES_CASTLING_KINGSIDE == EMPTY {
                    moves.push(Move {
                        from: Square::E1,
                        to: Square::G1,
                        variant: MoveType::CastleKingside,
                    })
                }
            },

            Color::Black => {
                if board.castling_rights.black_queenside && occupied & BLACK_TRAVERSES_CASTLING_QUEENSIDE == EMPTY {
                    moves.push(Move {
                        from: Square::E8,
                        to: Square::C8,
                        variant: MoveType::CastleQueenside,
                    })
                }

                if board.castling_rights.black_kingside && occupied & BLACK_TRAVERSES_CASTLING_KINGSIDE == EMPTY {
                    moves.push(Move {
                        from: Square::E8,
                        to: Square::G8,
                        variant: MoveType::CastleKingside,
                    })
                }
            },
        }

        moves
    }
}

#[cfg(test)]
mod tests {
    use crate::{board::PieceType, test_utils::format_pretty_list};
    use super::*;

    /// Asserts that moves contains m
    fn should_contain(moves: &Vec<Move>, m: &Move) {
        assert!(
            moves.contains(m),
            "The valid move {} was not generated! Available {}",
            m,
            format_pretty_list(moves)
        );
    }

    /// Asserts that moves doesn't contain m
    fn shouldnt_contain(moves: &Vec<Move>, m: &Move) {
        assert!(
            !moves.contains(m),
            "The invalid move {} was generated! Available {}",
            m,
            format_pretty_list(moves)
        );
    }

    #[test]
    fn white_castling_kingside() {
        let board = Board::from_fen("r2qkbnr/pp1b1ppp/2n1p3/1BppP3/3P4/5N2/PPP2PPP/RNBQK2R w KQkq - 4 6").unwrap();
        let castle_kingside = Move::new(Square::E1, Square::G1, &board);
        let castle_queenside = Move::new(Square::E1, Square::C1, &board);

        let moves = King(castle_kingside.from).psuedo_legal_moves(&board);
        should_contain(&moves, &castle_kingside);
        shouldnt_contain(&moves, &castle_queenside);
    }

    #[test]
    fn double_bongcloud() {
        let mut board = Board::default();

        for m in [
            Move::new(Square::E2, Square::E4, &board),
            Move::new(Square::E7, Square::E5, &board),
            Move::new(Square::E1, Square::E2, &board),
            Move::new(Square::E8, Square::E7, &board),
            Move::new(Square::E2, Square::D3, &board),
            Move::new(Square::E7, Square::F6, &board),
        ] {
            if board.determine_piece(m.from) == Some(PieceType::King) {
                let moves = King(m.from).psuedo_legal_moves(&board);
                should_contain(&moves, &m);
            }
            board = m.make(&board);
        }
    }
}
