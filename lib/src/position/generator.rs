use rand::{Rng, RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    bitboard::{BitBoard, EMPTY},
    position::{game::Game, legality::GameValidator},
    square::Square,
};

/// Generates random fens for testing Game
pub struct GameGenerator {
    rng: SmallRng,
}

impl GameGenerator {
    /// Creates a GameGenerator with a random seed
    pub fn unseeded() -> GameGenerator {
        let mut rng = rand::rng();
        let seed = rng.next_u32();
        GameGenerator::seeded(seed)
    }

    /// Creates a GameGenerator from the specified seed
    pub fn seeded(seed: u32) -> GameGenerator {
        let rng = SmallRng::seed_from_u64(seed.into());
        GameGenerator { rng }
    }

    /// Creates a new game that is not checked for legality
    pub fn next_maybe_legal_game(&mut self) -> Game {
        let mut game = Game::empty();

        let num_pieces = self.rng.random_range(2..33);
        let mut unidentified_pieces = self.next_bitboard_with_n_bits_set(num_pieces);

        macro_rules! identify {
            ($bb:ident) => {{
                let b = self.next_bitboard_with_n_bits_set_from_valid_area(1, unidentified_pieces);
                game.$bb |= b;
                unidentified_pieces ^= b;
            }};
        }

        identify!(white_kings);
        identify!(black_kings);

        while unidentified_pieces.popcnt() > 0 {
            let piece_chance: u8 = self.rng.random_range(1..=140);

            if piece_chance <= 70 {
                if piece_chance >= 35 {
                    identify!(white_pawns);
                } else if piece_chance >= 25 {
                    identify!(white_knights);
                } else if piece_chance >= 15 {
                    identify!(white_bishops);
                } else if piece_chance >= 6 {
                    identify!(white_rooks);
                } else {
                    identify!(white_queens);
                }
            } else if piece_chance >= 105 {
                identify!(black_pawns);
            } else if piece_chance >= 95 {
                identify!(black_knights);
            } else if piece_chance >= 85 {
                identify!(black_bishops);
            } else if piece_chance >= 76 {
                identify!(black_rooks);
            } else {
                identify!(black_queens);
            }
        }

        game.initialize();
        game
    }

    /// Generates maybe legal games and returns the first legal game found
    pub fn next_legal_game(&mut self, validator: &GameValidator) -> Game {
        loop {
            let game = self.next_maybe_legal_game();
            if validator.validate(&game) {
                return game;
            }
        }
    }

    /// Generates a random bitboard with the specified number of bits set. Only sets bits from the
    /// valid area specified.
    fn next_bitboard_with_n_bits_set_from_valid_area(
        &mut self,
        n: u8,
        valid: BitBoard,
    ) -> BitBoard {
        let max_bits = valid.popcnt();
        let requested_bits = n.into();
        let num_bits = if requested_bits > max_bits {
            max_bits
        } else {
            requested_bits
        };

        let mut board = EMPTY;
        while board.popcnt() < num_bits {
            let sq = unsafe { Square::new_unchecked(self.rng.random_range(0..64)) };
            let sqbb = BitBoard::from_square(sq);
            if valid.has_square(sqbb) {
                board |= sqbb;
            }
        }
        board
    }

    /// Generates a random bitboard with the specified number of bits set
    fn next_bitboard_with_n_bits_set(&mut self, n: u8) -> BitBoard {
        let valid = !EMPTY;
        self.next_bitboard_with_n_bits_set_from_valid_area(n, valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitboard_with_n_bits_set() {
        let mut gg = GameGenerator::unseeded();
        let expected = 23;
        let actual = gg.next_bitboard_with_n_bits_set(expected).popcnt() as u8;
        assert_eq!(actual, expected);
    }
}
