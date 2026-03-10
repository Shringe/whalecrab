use crate::{
    bitboard::{BitBoard, EMPTY},
    castling,
    file::File,
    game::Game,
    movegen::{
        moves::{Move, targets_to_moves},
        pieces::piece::PieceColor,
    },
    square::{ALL_DIRECTIONS, Square},
};

use super::piece::PieceMoveInfo;

impl Square {
    /// King safety not considered.
    pub fn king_psuedo_legal_moves(&self, game: &Game) -> Vec<Move> {
        targets_to_moves(self.king_psuedo_legal_targets(game).targets, *self, game)
    }

    pub fn king_psuedo_legal_targets(&self, game: &Game) -> PieceMoveInfo {
        let mut moveinfo = PieceMoveInfo::default();

        let enemy_or_empty = !*game.get_occupied(&game.turn);

        let sqbb = BitBoard::from_square(*self);
        let not_a_file = !File::A.mask();
        let not_h_file = !File::H.mask();

        let left = (sqbb >> BitBoard(1)) & not_h_file;
        let right = (sqbb << BitBoard(1)) & not_a_file;
        let middle_three = left | sqbb | right;

        let attacks =
            (middle_three >> BitBoard(8)) | (middle_three << BitBoard(8)) | (middle_three ^ sqbb);

        moveinfo.attacks |= attacks;
        moveinfo.targets |= attacks & enemy_or_empty;

        let occupied = &game.occupied;
        match game.turn {
            PieceColor::White => {
                if game.castling_rights.white_queenside
                    && occupied & castling::WHITE_CASTLE_QUEENSIDE_NEEDS_CLEAR == EMPTY
                {
                    moveinfo.targets |= castling::WHITE_CASTLE_QUEENSIDE_KING_TO_BB;
                }
                if game.castling_rights.white_kingside
                    && occupied & castling::WHITE_CASTLE_KINGSIDE_NEEDS_CLEAR == EMPTY
                {
                    moveinfo.targets |= castling::WHITE_CASTLE_KINGSIDE_KING_TO_BB;
                }
            }
            PieceColor::Black => {
                if game.castling_rights.black_queenside
                    && occupied & castling::BLACK_CASTLE_QUEENSIDE_NEEDS_CLEAR == EMPTY
                {
                    moveinfo.targets |= castling::BLACK_CASTLE_QUEENSIDE_KING_TO_BB;
                }
                if game.castling_rights.black_kingside
                    && occupied & castling::BLACK_CASTLE_KINGSIDE_NEEDS_CLEAR == EMPTY
                {
                    moveinfo.targets |= castling::BLACK_CASTLE_KINGSIDE_KING_TO_BB;
                }
            }
        }

        moveinfo
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
            let frombb = BitBoard::from_square(from);
            if matches!(game.determine_piece(&frombb), Some((PieceType::King, _))) {
                let moves = from.king_psuedo_legal_moves(&game);
                should_generate(&moves, &m);
            }
            game.play(&m);
        }
    }
}
