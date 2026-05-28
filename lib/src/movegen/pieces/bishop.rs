use crate::{
    bitboard::{BitBoard, EMPTY},
    file::File,
    movegen::moves::{Move, attacks_to_moves, push_attacks_to_moves_with_occupied},
    position::game::Game,
    rank::Rank,
    square::{Direction, Square},
    vectors::Vector,
};

use super::piece::PieceMoveInfo;

pub const MAXIMUM_MOVE_COUNT: u32 = 13;

pub const DIRECTIONS: [Direction; 4] = [
    Direction::NorthEast,
    Direction::SouthEast,
    Direction::NorthWest,
    Direction::SouthWest,
];

pub fn magic_attacks(sq: Square, occupied: BitBoard) -> BitBoard {
    let bishop = &magics::bishops::BISHOPS[sq.index()];
    let key = (((occupied.to_int() & bishop.mask).wrapping_mul(bishop.magic))
        >> (magics::bishops::SHIFT as u64)) as usize;
    BitBoard::new(bishop.attacks[key])
}

pub fn push_psuedo_legal_moves<V: Vector<Move>>(
    moves: &mut V,
    game: &Game,
    bishop: BitBoard,
    kingless_bb: BitBoard,
    enemy_occupied: BitBoard,
) {
    for sq in bishop {
        let attacks = magic_attacks(sq, kingless_bb);
        push_attacks_to_moves_with_occupied(moves, attacks, sq, game, enemy_occupied);
    }
}

impl Square {
    pub fn bishop_psuedo_legal_attacks(&self, game: &Game) -> BitBoard {
        let color = game.piece_lookup(*self).map(|p| p.1).unwrap_or(game.turn);
        let blockers = game.occupied ^ *game.get_king(color.opponent());
        magic_attacks(*self, blockers)
    }

    pub fn bishop_psuedo_legal_moves(&self, game: &Game) -> Vec<Move> {
        attacks_to_moves(self.bishop_psuedo_legal_attacks(game), *self, game)
    }

    pub fn bishop_psuedo_legal_targets(&self, game: &Game) -> PieceMoveInfo {
        self.rays(&DIRECTIONS, game)
    }

    /// Generates a list of rook targets considering blockers
    pub fn bishop_attacks_with_blockers(self, blockers: BitBoard) -> BitBoard {
        let mut out = EMPTY;
        for direction in DIRECTIONS {
            out |= self.ray_with_blockers(direction, blockers);
        }
        out
    }

    /// Generates bishop attacks within the bounds of a magic mask
    pub fn masked_bishop_attacks(self) -> BitBoard {
        let mut out = EMPTY;

        unsafe {
            self.custom_ray(&mut out, |c| {
                (c.get_rank() < Rank::Seventh && c.get_file() > File::B)
                    .then(|| c.uleft_unchecked())
            });
            self.custom_ray(&mut out, |c| {
                (c.get_rank() < Rank::Seventh && c.get_file() < File::G)
                    .then(|| c.uright_unchecked())
            });
            self.custom_ray(&mut out, |c| {
                (c.get_rank() > Rank::Second && c.get_file() > File::B).then(|| c.dleft_unchecked())
            });
            self.custom_ray(&mut out, |c| {
                (c.get_rank() > Rank::Second && c.get_file() < File::G)
                    .then(|| c.dright_unchecked())
            });
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use crate::{movegen::pieces::piece::PieceType, test_utils::format_pretty_list};

    use super::*;

    #[test]
    fn white_bishop_can_move_around() {
        let mut game = Game::default();

        for (from, to) in [
            (Square::G2, Square::G4),
            (Square::G8, Square::F6),
            (Square::F1, Square::G2),
            (Square::F6, Square::G8),
            (Square::G2, Square::C6),
            (Square::G8, Square::F6),
            (Square::C6, Square::G2),
            (Square::F6, Square::G8),
            (Square::G2, Square::F1),
        ] {
            let m = Move::infer(from, to, &game);
            if matches!(game.piece_lookup(from), Some((PieceType::Bishop, _))) {
                let moves = m.from(game.turn).bishop_psuedo_legal_moves(&game);
                assert!(
                    moves.contains(&m),
                    "The move {} not be found naturally! Available {}",
                    m,
                    format_pretty_list(&moves)
                );
            }
            game.play(&m);
        }
    }

    #[test]
    fn masked_bishop_attacks() {
        let sq = Square::E1;
        let expected = BitBoard::from_square_vec(vec![
            // Up left
            Square::D2,
            Square::C3,
            Square::B4,
            // Up right
            Square::F2,
            Square::G3,
        ]);
        let actual = sq.masked_bishop_attacks();
        assert_eq!(actual, expected);
    }
}
