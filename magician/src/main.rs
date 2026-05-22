use rand::{RngExt, distr::uniform::SampleRange};
use whalecrab_lib::{
    bitboard::{BitBoard, EMPTY},
    position::generator::GameGenerator,
    square::Square,
};

fn generate_rook_blockers_and_attackers(
    sq: Square,
    mask: BitBoard,
) -> ([(BitBoard, BitBoard); MagicRook::NUM_SUBSETS], usize) {
    let mut baa = [(EMPTY, EMPTY); MagicRook::NUM_SUBSETS];

    let mut subset = mask;
    for i in 0..MagicRook::NUM_SUBSETS {
        baa[i] = (subset, sq.rook_attacks_with_blockers(subset));
        if subset == EMPTY {
            return (baa, i + 1);
        }
        subset = BitBoard::new(subset.to_int() - 1) & mask;
    }

    unreachable!();
}

fn generate_magic_rooks<R: SampleRange<u8> + Clone>(
    grng: &mut GameGenerator,
    range: R,
) -> MagicRooks {
    let mut magicians = [MagicRook::EMPTY; 64];

    for sq in Square::ALL_SQUARES {
        let mask = sq.rook_mask();
        let (baa, len) = generate_rook_blockers_and_attackers(sq, mask);

        loop {
            let mut rook = MagicRook::EMPTY;
            rook.magic = next_magic(grng, range.clone());
            let valid =
                validate_magic(&mut rook.attacks, rook.magic, &baa[..len], MagicRook::SHIFT);

            if valid {
                magicians[sq.index()] = rook;
                break;
            }
        }
    }

    magicians
}
fn next_magic<R: SampleRange<u8>>(grng: &mut GameGenerator, range: R) -> u64 {
    let num_bits = grng.rng.random_range(range);
    let magicbb = grng.next_bitboard_with_n_bits_set(num_bits);
    magicbb.to_int()
}

fn validate_magic(
    attack_table: &mut [BitBoard],
    magic: u64,
    baa: &[(BitBoard, BitBoard)],
    shift: usize,
) -> bool {
    for (blocker, attacker) in baa {
        let index = blocker.to_int().wrapping_mul(magic) >> shift;
        let entry = &mut attack_table[index as usize];
        if *entry == EMPTY {
            *entry = *attacker;
        } else if *entry != *attacker {
            return false;
        }
    }
    true
}

type MagicRooks = [MagicRook; 64];

struct MagicRook {
    attacks: [BitBoard; MagicRook::NUM_SUBSETS],
    mask: BitBoard,
    magic: u64,
}

impl MagicRook {
    const NUM_BITS: u8 = 12;
    const NUM_SUBSETS: usize = 1 << MagicRook::NUM_BITS as usize;
    const SHIFT: usize = 64 - MagicRook::NUM_BITS as usize;

    const EMPTY: MagicRook = MagicRook {
        attacks: [EMPTY; MagicRook::NUM_SUBSETS],
        mask: EMPTY,
        magic: 0,
    };

    fn attacks(&self, occupied: BitBoard) -> BitBoard {
        let key =
            (((occupied & self.mask) * self.magic) >> (MagicRook::SHIFT as u64)).to_int() as usize;
        self.attacks[key]
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    use std::hash::{DefaultHasher, Hash, Hasher};

    use function_name::named;

    fn seed_from_function_name(name: &str) -> u32 {
        let mut hasher = DefaultHasher::default();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        hash.try_into().unwrap_or((hash & u32::MAX as u64) as u32)
    }

    macro_rules! seed {
        () => {
            GameGenerator::seeded(seed_from_function_name(function_name!()))
        };
    }

    #[test]
    #[named]
    fn rook_attacks_on_empty_board() {
        let mut grng = seed!();

        let rooks = generate_magic_rooks(&mut grng, 12..=12);
        let sq = Square::E8;
        let expected = sq.rook_attacks_with_blockers(EMPTY);
        let actual = rooks[sq.index()].attacks(EMPTY);
        assert_eq!(actual, expected);
    }
}
