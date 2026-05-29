use std::{ops::RangeInclusive, sync::OnceLock};

use whalecrab_lib::{bitboard::BitBoard, position::generator::GameGenerator, square::Square};

use crate::{BitRange, generate_blockers_and_attackers, next_magic, validate_magic};

pub fn generate_rook_blockers_and_attackers(
    sq: Square,
    mask: u64,
) -> ([(u64, u64); MagicRook::NUM_SUBSETS], usize) {
    generate_blockers_and_attackers(mask, |bb| sq.rook_attacks_with_blockers(bb))
}

pub fn generate_magic_rooks_owned<R: BitRange>(grng: &mut GameGenerator, range: R) -> MagicRooks {
    let mut magicians = [MagicRook::EMPTY; 64];

    for sq in Square::ALL_SQUARES {
        let mask = sq.masked_rook_attacks().to_int();
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

pub fn generate_magic_rooks<R: BitRange>(
    grng: &mut GameGenerator,
    range: R,
) -> &'static MagicRooks {
    ROOKS.get_or_init(|| generate_magic_rooks_owned(grng, range))
}

static ROOKS: OnceLock<MagicRooks> = OnceLock::new();

pub type MagicRooks = [MagicRook; 64];

pub struct MagicRook {
    attacks: [u64; MagicRook::NUM_SUBSETS],
    mask: u64,
    magic: u64,
}

impl std::fmt::Debug for MagicRook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.debug_struct("MagicRook")
                // .field("attacks", &self.attacks)
                .field("mask", &self.mask)
                .field("magic", &self.magic)
                .finish()
        } else {
            f.debug_struct("MagicRook")
                .field("attacks", &self.attacks)
                .field("mask", &self.mask)
                .field("magic", &self.magic)
                .finish()
        }
    }
}

impl MagicRook {
    pub const BIT_RANGE: RangeInclusive<u8> = Self::NUM_BITS..=Self::NUM_BITS;
    const NUM_BITS: u8 = 12;
    const NUM_SUBSETS: usize = 1 << Self::NUM_BITS as usize;
    const SHIFT: usize = 64 - Self::NUM_BITS as usize;

    const EMPTY: Self = Self {
        attacks: [0; Self::NUM_SUBSETS],
        mask: 0,
        magic: 0,
    };

    pub fn attacks(&self, occupied: BitBoard) -> BitBoard {
        let key = (((occupied & self.mask).to_int().wrapping_mul(self.magic))
            >> (MagicRook::SHIFT as u64)) as usize;
        BitBoard::new(self.attacks[key])
    }

    pub fn embed(&self, source: &mut String) {
        source.push_str("MagicRook{attacks:[");
        for pattern in self.attacks {
            source.push_str(pattern.to_string().as_str());
            source.push(',');
        }
        let _ = source.pop();

        source.push_str("],mask:");
        source.push_str(self.mask.to_string().as_str());
        source.push_str(",magic:");
        source.push_str(self.magic.to_string().as_str());
        source.push('}');
    }
}

pub fn embedded_magic_rook_file(rooks: &MagicRooks) -> String {
    let mut source = String::with_capacity(std::mem::size_of::<MagicRooks>().saturating_mul(4));
    let types = format! {
"pub const NUM_BITS: u8 = {};
pub const NUM_SUBSETS: usize = 1 << NUM_BITS as usize;
pub const SHIFT: usize = 64 - NUM_BITS as usize;

pub type MagicRooks = [MagicRook; 64];

pub struct MagicRook {{
    pub attacks: [u64; NUM_SUBSETS],
    pub mask: u64,
    pub magic: u64,
}}

", MagicRook::NUM_BITS
    };

    source.push_str(types.as_str());
    source.push_str("pub static ROOKS: MagicRooks = [");
    embed_magic_rooks(&mut source, rooks);
    source.push_str("];");

    source
}

pub fn embed_magic_rooks(source: &mut String, rooks: &MagicRooks) {
    for rook in rooks.iter() {
        rook.embed(source);
        source.push(',')
    }
    let _ = source.pop();
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::hash::{DefaultHasher, Hash, Hasher};

    use function_name::named;
    use rand::RngExt;
    use whalecrab_lib::bitboard::EMPTY;

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
    #[ignore]
    fn canary_should_not_overflow_stack_four_tables() {
        let mut grng = seed!();
        stack_recurse(
            &mut grng,
            |grng| generate_magic_rooks_owned(grng, 1..100),
            4,
        );
    }

    #[test]
    #[named]
    fn serialize() {
        let mut grng = seed!();
        let rooks = generate_magic_rooks(&mut grng, 12..=12);
        let mut source = String::with_capacity(std::mem::size_of::<MagicRooks>());
        embed_magic_rooks(&mut source, rooks);
    }
}
