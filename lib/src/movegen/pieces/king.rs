use crate::{
    bitboard::{BitBoard, EMPTY},
    file::File,
    movegen::{
        moves::{Move, push_attacks_to_moves_with_occupied},
        pieces::piece::PieceColor,
    },
    position::{
        castling::{self, CastleSide},
        game::Game,
    },
    square::Square,
    vectors::Vector,
};

use super::piece::PieceMoveInfo;

pub const MAXIMUM_MOVE_COUNT: u32 = 8;

static ATTACKS: [BitBoard; 64] = {
    let mut table = [EMPTY; 64];
    let mut n = 0;
    while n < 64 {
        let sq = Square::new(n);
        let sqbb = BitBoard::from_square(sq);
        table[sq.index()] = psuedo_legal_attacks(sqbb);
        n += 1;
    }
    table
};

const fn psuedo_legal_attacks(sqbb: BitBoard) -> BitBoard {
    let left = (sqbb.to_int() >> 1) & !File::H.mask().to_int();
    let right = (sqbb.to_int() << 1) & !File::A.mask().to_int();
    let middle_three = left | sqbb.to_int() | right;
    BitBoard::new((middle_three >> 8) | (middle_three << 8) | (middle_three ^ sqbb.to_int()))
}

pub fn push_psuedo_legal_moves<V: Vector<Move>>(
    moves: &mut V,
    game: &Game,
    kings: BitBoard,
    enemy_occupied: BitBoard,
) {
    for sq in kings {
        push_attacks_to_moves_with_occupied(moves, attacks(sq), sq, game, enemy_occupied);
    }
}

pub fn push_psuedo_legal_castling_moves_white<V: Vector<Move>>(moves: &mut V, game: &Game) {
    if game.castling_rights.white_queenside()
        && game.occupied & castling::WHITE_CASTLE_QUEENSIDE_NEEDS_CLEAR == EMPTY
    {
        moves.push(Move::Castle {
            side: CastleSide::Queenside,
        });
    }
    if game.castling_rights.white_kingside()
        && game.occupied & castling::WHITE_CASTLE_KINGSIDE_NEEDS_CLEAR == EMPTY
    {
        moves.push(Move::Castle {
            side: CastleSide::Kingside,
        });
    }
}

pub fn push_psuedo_legal_castling_moves_black<V: Vector<Move>>(moves: &mut V, game: &Game) {
    if game.castling_rights.black_queenside()
        && game.occupied & castling::BLACK_CASTLE_QUEENSIDE_NEEDS_CLEAR == EMPTY
    {
        moves.push(Move::Castle {
            side: CastleSide::Queenside,
        });
    }
    if game.castling_rights.black_kingside()
        && game.occupied & castling::BLACK_CASTLE_KINGSIDE_NEEDS_CLEAR == EMPTY
    {
        moves.push(Move::Castle {
            side: CastleSide::Kingside,
        });
    }
}

pub fn attacks(sq: Square) -> BitBoard {
    ATTACKS[sq.index()]
}

impl Square {
    /// King safety not considered.
    pub fn king_psuedo_legal_moves(self, game: &Game) -> Vec<Move> {
        self.lazy_king_pseudo_legal_moves(game).collect()
    }

    pub fn king_psuedo_legal_targets(self, game: &Game) -> PieceMoveInfo {
        let mut moveinfo = PieceMoveInfo::default();

        let enemy_or_empty = !*game.get_occupied(&game.turn);

        let sqbb = BitBoard::from_square(self);
        let not_a_file = !File::A.mask();
        let not_h_file = !File::H.mask();

        let left = (sqbb >> 1) & not_h_file;
        let right = (sqbb << 1) & not_a_file;
        let middle_three = left | sqbb | right;

        let attacks = (middle_three >> 8) | (middle_three << 8) | (middle_three ^ sqbb);

        moveinfo.attacks |= attacks;
        moveinfo.targets |= attacks & enemy_or_empty;

        let occupied = game.occupied;
        match game.turn {
            PieceColor::White => {
                if game.castling_rights.white_queenside()
                    && occupied & castling::WHITE_CASTLE_QUEENSIDE_NEEDS_CLEAR == EMPTY
                {
                    moveinfo.targets |= castling::WHITE_CASTLE_QUEENSIDE_KING_TO_BB;
                }
                if game.castling_rights.white_kingside()
                    && occupied & castling::WHITE_CASTLE_KINGSIDE_NEEDS_CLEAR == EMPTY
                {
                    moveinfo.targets |= castling::WHITE_CASTLE_KINGSIDE_KING_TO_BB;
                }
            }
            PieceColor::Black => {
                if game.castling_rights.black_queenside()
                    && occupied & castling::BLACK_CASTLE_QUEENSIDE_NEEDS_CLEAR == EMPTY
                {
                    moveinfo.targets |= castling::BLACK_CASTLE_QUEENSIDE_KING_TO_BB;
                }
                if game.castling_rights.black_kingside()
                    && occupied & castling::BLACK_CASTLE_KINGSIDE_NEEDS_CLEAR == EMPTY
                {
                    moveinfo.targets |= castling::BLACK_CASTLE_KINGSIDE_KING_TO_BB;
                }
            }
        }

        moveinfo
    }

    pub fn lazy_king_pseudo_legal_moves(self, game: &Game) -> impl Iterator<Item = Move> {
        let color = game.turn;

        let (enemy_occupied, castling_moves) = match color {
            PieceColor::White => (
                game.black_occupied,
                [
                    game.can_white_castle_queenside().then_some(Move::Castle {
                        side: CastleSide::Queenside,
                    }),
                    game.can_white_castle_kingside().then_some(Move::Castle {
                        side: CastleSide::Kingside,
                    }),
                ]
                .into_iter()
                .flatten(),
            ),
            PieceColor::Black => (
                game.white_occupied,
                [
                    game.can_black_castle_queenside().then_some(Move::Castle {
                        side: CastleSide::Queenside,
                    }),
                    game.can_black_castle_kingside().then_some(Move::Castle {
                        side: CastleSide::Kingside,
                    }),
                ]
                .into_iter()
                .flatten(),
            ),
        };

        let attacks = attacks(self);
        let captures = attacks & enemy_occupied;
        let walks = attacks & !game.occupied;

        let capture_moves = captures.into_iter().map(move |sq| Move::Normal {
            from: self,
            to: sq,
            capture: Some(unsafe { game.piece_lookup(sq).unwrap_unchecked().0 }),
        });

        let walk_moves = walks.into_iter().map(move |sq| Move::Normal {
            from: self,
            to: sq,
            capture: None,
        });

        (castling_moves).chain(capture_moves).chain(walk_moves)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        movegen::pieces::piece::PieceType,
        test_utils::{should_generate, shouldnt_generate},
    };

    #[test]
    fn white_sees_castling_kingside() {
        let fen = "r2qkbnr/pp1b1ppp/2n1p3/1BppP3/3P4/5N2/PPP2PPP/RNBQK2R w KQkq - 4 6";
        let game = Game::from_fen(fen).unwrap();
        let moves = Square::E1.king_psuedo_legal_moves(&game);
        should_generate(
            &moves,
            &Move::Castle {
                side: castling::CastleSide::Kingside,
            },
        );
        shouldnt_generate(
            &moves,
            &Move::Castle {
                side: castling::CastleSide::Queenside,
            },
        );
    }

    #[test]
    fn black_sees_castling_queenside() {
        let fen = "r3kbnr/pp1bqppp/2n1p3/1BppP3/3P4/5N2/PPP2PPP/RNBQK2R b KQkq - 5 6";
        let game = Game::from_fen(fen).unwrap();
        let moves = Square::E8.king_psuedo_legal_moves(&game);
        should_generate(
            &moves,
            &Move::Castle {
                side: castling::CastleSide::Queenside,
            },
        );
        shouldnt_generate(
            &moves,
            &Move::Castle {
                side: castling::CastleSide::Kingside,
            },
        );
    }

    #[test]
    fn black_sees_castling_queenside_fast() {
        let fen = "r3kbnr/pp1bqppp/2n1p3/1BppP3/3P4/5N2/PPP2PPP/RNBQK2R b KQkq - 5 6";
        let game = Game::from_fen(fen).unwrap();

        let mut expected = PieceMoveInfo::default();
        expected.targets.set(Square::D8);
        expected.targets.set(Square::C8);
        expected.attacks.set(Square::D8);
        expected.attacks.set(Square::F8);
        expected.attacks.set(Square::F7);
        expected.attacks.set(Square::D7);
        expected.attacks.set(Square::E7);

        let actual = Square::E8.king_psuedo_legal_targets(&game);
        assert_eq!(actual, expected);
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
            let m = Move::infer(from, to, &game);
            if matches!(game.piece_lookup(from), Some((PieceType::King, _))) {
                let moves = from.king_psuedo_legal_moves(&game);
                should_generate(&moves, &m);
            }
            game.play(&m);
        }
    }
}
