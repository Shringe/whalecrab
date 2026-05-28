use crate::{
    bitboard::{BitBoard, EMPTY},
    file::File,
    movegen::{
        moves::{Move, attacks_to_moves, push_attacks_to_moves_with_occupied},
        pieces::piece::PieceMoveInfo,
    },
    position::game::Game,
    square::Square,
    vectors::Vector,
};

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
    let bb = sqbb.to_int();

    let not_a = !File::A.mask().to_int();
    let not_ab = !File::A.mask().to_int() & !File::B.mask().to_int();
    let not_h = !File::H.mask().to_int();
    let not_gh = !File::G.mask().to_int() & !File::H.mask().to_int();

    let attacks = (bb << 17) & not_a
        | (bb << 15) & not_h
        | (bb << 10) & not_ab
        | (bb << 6) & not_gh
        | (bb >> 17) & not_h
        | (bb >> 15) & not_a
        | (bb >> 10) & not_gh
        | (bb >> 6) & not_ab;

    BitBoard::new(attacks)
}

pub fn push_psuedo_legal_moves<V: Vector<Move>>(
    moves: &mut V,
    game: &Game,
    knights: BitBoard,
    enemy_occupied: BitBoard,
) {
    for sq in knights {
        push_attacks_to_moves_with_occupied(moves, attacks(sq), sq, game, enemy_occupied);
    }
}

pub fn attacks(sq: Square) -> BitBoard {
    ATTACKS[sq.index()]
}

impl Square {
    pub fn knight_psuedo_legal_moves(self, game: &Game) -> Vec<Move> {
        attacks_to_moves(attacks(self), self, game)
    }

    pub fn knight_psuedo_legal_targets(self, game: &Game) -> PieceMoveInfo {
        let mut moveinfo = PieceMoveInfo::default();
        let enemy_or_empty = !*game.get_occupied(&game.turn);
        let attacks = attacks(self);

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
