use crate::{
    bitboard::{BitBoard, EMPTY},
    board::Color,
    castling,
    game::Game,
    movegen::moves::{Move, MoveType},
    square::{Square, ALL_DIRECTIONS},
};

use super::piece::Piece;

pub struct King(pub Square);

impl Piece for King {
    /// King safety not considered.
    fn psuedo_legal_moves(&self, game: &mut Game) -> Vec<Move> {
        let mut moves = Vec::new();

        let friendly = game.position.turn;
        let enemy = friendly.opponent();

        for d in ALL_DIRECTIONS {
            if let Some(sq) = self.0.walk(&d) {
                let sqbb = BitBoard::from_square(sq);
                let attack_bitboard = game.get_attacks_mut(&friendly);
                attack_bitboard.set(sq);

                if let Some((piece, color)) = game.determine_piece(&sqbb) {
                    if color == enemy {
                        moves.push(Move {
                            from: self.0,
                            to: sq,
                            variant: MoveType::Capture(piece),
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

        let occupied = &game.occupied;
        match game.position.turn {
            Color::White => {
                if game.position.castling_rights.white_queenside
                    && occupied & castling::WHITE_CASTLE_QUEENSIDE_NEEDS_CLEAR == EMPTY
                {
                    moves.push(castling::WHITE_CASTLES_QUEENSIDE);
                }

                if game.position.castling_rights.white_kingside
                    && occupied & castling::WHITE_CASTLE_KINGSIDE_NEEDS_CLEAR == EMPTY
                {
                    moves.push(castling::WHITE_CASTLES_KINGSIDE);
                }
            }

            Color::Black => {
                if game.position.castling_rights.black_queenside
                    && occupied & castling::BLACK_CASTLE_QUEENSIDE_NEEDS_CLEAR == EMPTY
                {
                    moves.push(castling::BLACK_CASTLES_QUEENSIDE);
                }

                if game.position.castling_rights.black_kingside
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
        board::{Board, PieceType},
        test_utils::{should_generate, shouldnt_generate},
    };

    #[test]
    fn white_sees_castling_kingside() {
        let mut game = Game::from_position(
            Board::from_fen("r2qkbnr/pp1b1ppp/2n1p3/1BppP3/3P4/5N2/PPP2PPP/RNBQK2R w KQkq - 4 6")
                .unwrap(),
        );
        let moves = King(castling::WHITE_CASTLES_KINGSIDE.from).psuedo_legal_moves(&mut game);
        should_generate(&moves, &castling::WHITE_CASTLES_KINGSIDE);
        shouldnt_generate(&moves, &castling::WHITE_CASTLES_QUEENSIDE);
    }

    #[test]
    fn black_sees_castling_queenside() {
        let mut game = Game::from_position(
            Board::from_fen("r3kbnr/pp1bqppp/2n1p3/1BppP3/3P4/5N2/PPP2PPP/RNBQK2R b KQkq - 5 6")
                .unwrap(),
        );
        let moves = King(castling::BLACK_CASTLES_QUEENSIDE.from).psuedo_legal_moves(&mut game);
        should_generate(&moves, &castling::BLACK_CASTLES_QUEENSIDE);
        shouldnt_generate(&moves, &castling::BLACK_CASTLES_KINGSIDE);
    }

    #[test]
    fn double_bongcloud() {
        let mut game = Game::default();

        for (from, to) in [
            (Square::E2, Square::E4),
            (Square::E7, Square::E5),
            (Square::E1, Square::E2),
            (Square::E8, Square::E7),
            (Square::E2, Square::D3),
            (Square::E7, Square::F6),
        ] {
            let m = Move::new(from, to, &game.position);
            let frombb = BitBoard::from_square(m.from);
            if matches!(game.determine_piece(&frombb), Some((PieceType::King, _))) {
                let moves = King(m.from).psuedo_legal_moves(&mut game);
                should_generate(&moves, &m);
            }
            game.play(&m);
        }
    }
}
