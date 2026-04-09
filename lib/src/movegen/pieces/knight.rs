use crate::{
    bitboard::BitBoard,
    file::File,
    position::game::Game,
    movegen::{
        moves::{Move, targets_to_moves},
        pieces::piece::PieceMoveInfo,
    },
    square::Square,
};

impl Square {
    pub fn knight_psuedo_legal_moves(&self, game: &Game) -> Vec<Move> {
        targets_to_moves(self.knight_psuedo_legal_targets(game).targets, *self, game)
    }

    pub fn knight_psuedo_legal_targets(&self, game: &Game) -> PieceMoveInfo {
        let mut moveinfo = PieceMoveInfo::default();

        let sqbb = BitBoard::from_square(*self);
        let enemy_or_empty = !*game.get_occupied(&game.turn);

        let not_file_a = !File::A.mask();
        let not_file_b = !File::B.mask();
        let not_file_g = !File::G.mask();
        let not_file_h = !File::H.mask();

        let attacks = (sqbb << 17) & not_file_a
            | (sqbb << 15) & not_file_h
            | (sqbb << 10) & not_file_a & not_file_b
            | (sqbb << 6) & not_file_g & not_file_h
            | (sqbb >> 6) & not_file_a & not_file_b
            | (sqbb >> 10) & not_file_g & not_file_h
            | (sqbb >> 15) & not_file_a
            | (sqbb >> 17) & not_file_h;

        moveinfo.attacks = attacks;
        moveinfo.targets = attacks & enemy_or_empty;

        moveinfo
    }
}

#[cfg(test)]
mod tests {
    use crate::movegen::pieces::piece::{PieceColor, PieceType};
    use crate::test_utils::format_pretty_list;

    use super::*;

    #[test]
    fn knight_cant_capture_en_passant() {
        let mut game = Game::default();
        let avoid = Move::CaptureEnPassant {
            from: Square::E5.get_file(),
        };
        for m in [
            Move::Normal {
                from: Square::G1,
                to: Square::F3,
                capture: None,
            },
            Move::Normal {
                from: Square::A7,
                to: Square::A5,
                capture: None,
            },
            Move::Normal {
                from: Square::F3,
                to: Square::E5,
                capture: None,
            },
            Move::CreateEnPassant {
                at: Square::C7.get_file(),
            },
        ] {
            game.play(&m);
        }

        let moves = avoid.from(game.turn).knight_psuedo_legal_moves(&game);
        assert!(!moves.contains(&avoid));
    }

    #[test]
    fn white_knight_captures_black_pawn() {
        let mut game = Game::default();
        let capture = dbg!(Move::Normal {
            from: Square::F5,
            to: Square::E7,
            capture: Some(PieceType::Pawn),
        });

        for m in [
            Move::Normal {
                from: Square::G1,
                to: Square::F3,
                capture: None,
            },
            Move::Normal {
                from: Square::A7,
                to: Square::A6,
                capture: None,
            },
            Move::Normal {
                from: Square::F3,
                to: Square::D4,
                capture: None,
            },
            Move::Normal {
                from: Square::A6,
                to: Square::A5,
                capture: None,
            },
            Move::Normal {
                from: Square::D4,
                to: Square::F5,
                capture: None,
            },
            Move::Normal {
                from: Square::A5,
                to: Square::A4,
                capture: None,
            },
        ] {
            if game.turn == PieceColor::White {
                let moves = m.from(game.turn).knight_psuedo_legal_moves(&game);
                assert!(
                    moves.contains(&m),
                    "Tried to make '{}' in order to set up the board, but it couldn't happen normally! The knight only sees: {}.",
                    m,
                    format_pretty_list(&moves)
                )
            }
            game.play(&m);
        }

        let moves = capture.from(game.turn).knight_psuedo_legal_moves(&game);

        assert!(
            moves.contains(&capture),
            "Knight did not generate expected capture move. Availabe: {}",
            format_pretty_list(&moves)
        );

        let knight_before = game.white_knights.popcnt();
        let pawns_before = game.black_pawns.popcnt();
        game.play(&capture);
        let knight_after = game.white_knights.popcnt();
        let pawns_after = game.black_pawns.popcnt();

        assert_eq!(knight_before, knight_after, "We lost the knight!");
        assert_eq!(
            pawns_before - 1,
            pawns_after,
            "The pawn is still standing knight!"
        );
    }
}
