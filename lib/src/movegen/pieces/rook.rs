use crate::{
    bitboard::{BitBoard, EMPTY},
    file::File,
    movegen::{
        moves::{Move, attacks_to_moves},
        pieces::piece::PieceColor,
    },
    position::game::Game,
    rank::Rank,
    square::{Direction, Square},
};

use super::piece::PieceMoveInfo;

pub const DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::South,
    Direction::East,
    Direction::West,
];

pub fn magic_rook_attacks(sq: Square, occupied: BitBoard) -> BitBoard {
    let rook = &magics::rooks::ROOKS[sq.index()];
    let key = (((occupied.to_int() & rook.mask).wrapping_mul(rook.magic))
        >> (magics::rooks::SHIFT as u64)) as usize;
    BitBoard::new(rook.attacks[key])
}

impl Square {
    pub fn rook_psuedo_legal_attacks(&self, game: &Game) -> BitBoard {
        let color = game.piece_lookup(*self).map(|p| p.1).unwrap_or(game.turn);
        let blockers = game.occupied ^ *game.get_king(color.opponent());
        magic_rook_attacks(*self, blockers)
    }

    pub fn rook_psuedo_legal_moves(&self, game: &Game) -> Vec<Move> {
        attacks_to_moves(self.rook_psuedo_legal_attacks(game), *self, game)
    }

    pub fn rook_psuedo_legal_targets(&self, game: &Game) -> PieceMoveInfo {
        self.rays(&DIRECTIONS, game)
    }

    /// Generates a list of rook targets considering blockers
    pub fn rook_attacks_with_blockers(self, blockers: BitBoard) -> BitBoard {
        let mut out = EMPTY;
        for direction in DIRECTIONS {
            out |= self.ray_with_blockers(direction, blockers);
        }
        out
    }

    /// Generates rook attacks within the bounds of a magic mask
    pub fn masked_rook_attacks(self) -> BitBoard {
        let mut out = EMPTY;

        unsafe {
            self.custom_ray(&mut out, |c| {
                (c.get_file() > File::B).then(|| c.left_unchecked())
            });
            self.custom_ray(&mut out, |c| {
                (c.get_file() < File::G).then(|| c.right_unchecked())
            });
            self.custom_ray(&mut out, |c| {
                (c.get_rank() < Rank::Seventh).then(|| c.up_unchecked())
            });
            self.custom_ray(&mut out, |c| {
                (c.get_rank() > Rank::Second).then(|| c.down_unchecked())
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
    fn white_rook_can_move_around() {
        let mut game = Game::default();

        for (from, to) in [
            (Square::A2, Square::A4),
            (Square::G8, Square::F6),
            (Square::A1, Square::A3),
            (Square::F6, Square::G8),
            (Square::A3, Square::H3),
            (Square::G8, Square::F6),
            (Square::H3, Square::A3),
            (Square::F6, Square::G8),
            (Square::A3, Square::A1),
        ] {
            let m = Move::infer(from, to, &game);
            if matches!(game.piece_lookup(from), Some((PieceType::Rook, _))) {
                let moves = m.from(game.turn).rook_psuedo_legal_moves(&game);
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
    fn masked_rook_attacks() {
        let sq = Square::E1;
        let expected = BitBoard::new(
            0b00000000_00010000_00010000_00010000_00010000_00010000_00010000_01101110,
        );
        let actual = sq.masked_rook_attacks();
        assert_eq!(actual, expected);
    }
}
