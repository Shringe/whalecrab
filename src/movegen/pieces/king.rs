use crate::{
    bitboard::EMPTY,
    board::{Board, Color},
    castling,
    movegen::moves::{Move, MoveType},
    square::{Square, ALL_DIRECTIONS},
};

use super::piece::Piece;

pub struct King(pub Square);

impl Piece for King {
    /// King safety not considered.
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
                if board.castling_rights.white_queenside
                    && occupied & castling::WHITE_CASTLE_QUEENSIDE_NEEDS_CLEAR == EMPTY
                {
                    moves.push(castling::WHITE_CASTLES_QUEENSIDE);
                }

                if board.castling_rights.white_kingside
                    && occupied & castling::WHITE_CASTLE_KINGSIDE_NEEDS_CLEAR == EMPTY
                {
                    moves.push(castling::WHITE_CASTLES_KINGSIDE);
                }
            }

            Color::Black => {
                if board.castling_rights.black_queenside
                    && occupied & castling::BLACK_CASTLE_QUEENSIDE_NEEDS_CLEAR == EMPTY
                {
                    moves.push(castling::BLACK_CASTLES_QUEENSIDE);
                }

                if board.castling_rights.black_kingside
                    && occupied & castling::BLACK_CASTLE_KINGSIDE_NEEDS_CLEAR == EMPTY
                {
                    moves.push(castling::BLACK_CASTLES_KINGSIDE);
                }
            }
        }

        moves
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        board::PieceType,
        test_utils::{should_generate, shouldnt_generate},
    };

    #[test]
    fn white_sees_castling_kingside() {
        let board =
            Board::from_fen("r2qkbnr/pp1b1ppp/2n1p3/1BppP3/3P4/5N2/PPP2PPP/RNBQK2R w KQkq - 4 6")
                .unwrap();
        let moves = King(castling::WHITE_CASTLES_KINGSIDE.from).psuedo_legal_moves(&board);
        should_generate(&moves, &castling::WHITE_CASTLES_KINGSIDE);
        shouldnt_generate(&moves, &castling::WHITE_CASTLES_QUEENSIDE);
    }

    #[test]
    fn black_sees_castling_queenside() {
        let board =
            Board::from_fen("r3kbnr/pp1bqppp/2n1p3/1BppP3/3P4/5N2/PPP2PPP/RNBQK2R b KQkq - 5 6")
                .unwrap();
        let moves = King(castling::BLACK_CASTLES_QUEENSIDE.from).psuedo_legal_moves(&board);
        should_generate(&moves, &castling::BLACK_CASTLES_QUEENSIDE);
        shouldnt_generate(&moves, &castling::BLACK_CASTLES_KINGSIDE);
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
                should_generate(&moves, &m);
            }
            board = m.make(&board);
        }
    }
}
