use std::{ops::RangeInclusive, sync::OnceLock};

use whalecrab_lib::{bitboard::BitBoard, position::generator::GameGenerator, square::Square};

use crate::{BitRange, generate_blockers_and_attackers, next_magic, validate_magic};

pub fn generate_bishop_blockers_and_attackers(
    sq: Square,
    mask: u64,
) -> ([(u64, u64); MagicBishop::NUM_SUBSETS], usize) {
    generate_blockers_and_attackers(mask, |bb| sq.bishop_attacks_with_blockers(bb))
}

pub fn generate_magic_bishops_owned<R: BitRange>(
    grng: &mut GameGenerator,
    range: R,
) -> MagicBishops {
    let mut magicians = [MagicBishop::EMPTY; 64];

    for sq in Square::ALL_SQUARES {
        let mask = sq.masked_bishop_attacks().to_int();
        let (baa, len) = generate_bishop_blockers_and_attackers(sq, mask);

        loop {
            let mut bishop = MagicBishop::EMPTY;
            bishop.mask = mask;
            bishop.magic = next_magic(grng, range.clone());
            let valid = validate_magic(
                &mut bishop.attacks,
                bishop.magic,
                &baa[..len],
                MagicBishop::SHIFT,
            );

            if valid {
                magicians[sq.index()] = bishop;
                break;
            }
        }
    }

    magicians
}

pub fn generate_magic_bishops<R: BitRange>(
    grng: &mut GameGenerator,
    range: R,
) -> &'static MagicBishops {
    BISHOPS.get_or_init(|| generate_magic_bishops_owned(grng, range))
}

static BISHOPS: OnceLock<MagicBishops> = OnceLock::new();

pub type MagicBishops = [MagicBishop; 64];

pub struct MagicBishop {
    attacks: [u64; MagicBishop::NUM_SUBSETS],
    mask: u64,
    magic: u64,
}

impl std::fmt::Debug for MagicBishop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.debug_struct("MagicBishop")
                // .field("attacks", &self.attacks)
                .field("mask", &self.mask)
                .field("magic", &self.magic)
                .finish()
        } else {
            f.debug_struct("MagicBishop")
                .field("attacks", &self.attacks)
                .field("mask", &self.mask)
                .field("magic", &self.magic)
                .finish()
        }
    }
}

impl MagicBishop {
    pub const BIT_RANGE: RangeInclusive<u8> = Self::NUM_BITS..=Self::NUM_BITS;
    const NUM_BITS: u8 = 9;
    const NUM_SUBSETS: usize = 1 << Self::NUM_BITS as usize;
    const SHIFT: usize = 64 - Self::NUM_BITS as usize;

    const EMPTY: Self = Self {
        attacks: [0; Self::NUM_SUBSETS],
        mask: 0,
        magic: 0,
    };

    pub fn attacks(&self, occupied: BitBoard) -> BitBoard {
        let key = (((occupied & self.mask).to_int().wrapping_mul(self.magic))
            >> (MagicBishop::SHIFT as u64)) as usize;
        BitBoard::new(self.attacks[key])
    }

    pub fn embed(&self, source: &mut String) {
        source.push_str("MagicBishop{attacks:[");
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

pub fn embedded_magic_bishop_file(bishops: &MagicBishops) -> String {
    let mut source = String::with_capacity(std::mem::size_of::<MagicBishops>().saturating_mul(4));
    let types = format! {
"pub const NUM_BITS: u8 = {};
pub const NUM_SUBSETS: usize = 1 << NUM_BITS as usize;
pub const SHIFT: usize = 64 - NUM_BITS as usize;

pub type MagicBishops = [MagicBishop; 64];

pub struct MagicBishop {{
    pub attacks: [u64; NUM_SUBSETS],
    pub mask: u64,
    pub magic: u64,
}}

", MagicBishop::NUM_BITS
    };

    source.push_str(types.as_str());
    source.push_str("pub static ROOKS: MagicBishops = [");
    embed_magic_bishops(&mut source, bishops);
    source.push_str("];");

    source
}

pub fn embed_magic_bishops(source: &mut String, bishops: &MagicBishops) {
    for bishop in bishops.iter() {
        bishop.embed(source);
        source.push(',')
    }
    let _ = source.pop();
}
