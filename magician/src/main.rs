use std::sync::OnceLock;

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

fn generate_magic_rooks_owned<R: SampleRange<u8> + Clone>(
    grng: &mut GameGenerator,
    range: R,
) -> MagicRooks {
    let mut magicians = [MagicRook::EMPTY; 64];

    for sq in Square::ALL_SQUARES {
        let mask = sq.masked_rook_attacks();
        let (baa, len) = generate_rook_blockers_and_attackers(sq, mask);

        loop {
            let mut rook = MagicRook::EMPTY;
            rook.mask = mask;
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

fn generate_magic_rooks<R: SampleRange<u8> + Clone>(
    grng: &mut GameGenerator,
    range: R,
) -> &'static MagicRooks {
    ROOKS.get_or_init(|| generate_magic_rooks_owned(grng, range))
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

static ROOKS: OnceLock<MagicRooks> = OnceLock::new();

type MagicRooks = [MagicRook; 64];

struct MagicRook {
    attacks: [BitBoard; MagicRook::NUM_SUBSETS],
    mask: BitBoard,
    magic: u64,
}

impl std::fmt::Debug for MagicRook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MagicRook")
            // .field("attacks", &self.attacks)
            .field("mask", &self.mask)
            .field("magic", &self.magic)
            .finish()
    }
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
        let key = (((occupied & self.mask).to_int().wrapping_mul(self.magic))
            >> (MagicRook::SHIFT as u64)) as usize;
        self.attacks[key]
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    use std::hash::{DefaultHasher, Hash, Hasher};

    use function_name::named;

    macro_rules! seed {
        () => {
            GameGenerator::seeded(seed_from_function_name(function_name!()))
        };
    }

    fn seed_from_function_name(name: &str) -> u32 {
        let mut hasher = DefaultHasher::default();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        hash.try_into().unwrap_or((hash & u32::MAX as u64) as u32)
    }

    fn stack_recurse<F, T, State>(state: &mut State, allocate: F, n: usize)
    where
        F: Fn(&mut State) -> T,
    {
        let _t = allocate(state);
        if n > 0 {
            stack_recurse(state, allocate, n - 1);
        }
    }

    #[test]
    #[named]
    fn rook_attacks_on_empty_board() {
        let mut grng = seed!();
        let rooks = generate_magic_rooks(&mut grng, 12..=12);

        for sq in Square::ALL_SQUARES {
            let blockers = EMPTY;
            let expected = sq.rook_attacks_with_blockers(blockers);
            let actual = rooks[sq.index()].attacks(blockers);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    #[named]
    fn rook_attacks_on_empty_board_with_blockers() {
        let mut grng = seed!();
        let rooks = generate_magic_rooks(&mut grng, 12..=12);
        // let rooks = generate_magic_rooks_owned(&mut grng, 12..=12);

        for sq in Square::ALL_SQUARES {
            let num_blockers = grng.rng.random_range(1..MagicRook::NUM_BITS);
            let blockers = grng.next_bitboard_with_n_bits_set(num_blockers);

            let expected = sq.rook_attacks_with_blockers(blockers);
            let actual = rooks[sq.index()].attacks(blockers);
            assert_eq!(
                actual,
                expected,
                "\n\nSquare: {}\nRook: {:#?}\nBlockers: {:#?}\nActual: {:#?}\nExpected: {:#?}\n\n",
                sq,
                rooks[sq.index()],
                blockers,
                actual,
                expected
            );
        }
    }

    #[test]
    #[named]
    #[ignore = "This is supposed to fail"]
    fn should_overflow_stack() {
        let mut grng = seed!();
        stack_recurse(
            &mut grng,
            |grng| generate_magic_rooks_owned(grng, 1..100),
            1000,
        );
    }

    #[test]
    #[named]
    fn should_not_overflow_stack_with_static_pointers() {
        let mut grng = seed!();
        stack_recurse(&mut grng, |grng| generate_magic_rooks(grng, 1..100), 1000);
    }

    #[test]
    #[named]
    fn should_not_overflow_stack_four_tables() {
        let mut grng = seed!();
        stack_recurse(
            &mut grng,
            |grng| generate_magic_rooks_owned(grng, 1..100),
            4,
        );
    }
}
