pub mod bishops;
pub mod rooks;

use rand::{RngExt, distr::uniform::SampleRange};
use whalecrab_lib::{bitboard::BitBoard, position::generator::GameGenerator};

pub trait BitRange: SampleRange<u8> + Clone {}
impl<T: SampleRange<u8> + Clone> BitRange for T {}

pub fn generate_blockers_and_attackers<const N: usize>(
    mask: u64,
    attacks_fn: impl Fn(BitBoard) -> BitBoard,
) -> ([(u64, u64); N], usize) {
    let mut baa = [(0, 0); N];

    let mut subset = mask;
    for i in 0..N {
        baa[i] = (subset, attacks_fn(BitBoard::new(subset)).to_int());
        if subset == 0 {
            return (baa, i + 1);
        }
        subset = (subset - 1) & mask;
    }

    unreachable!()
}

pub fn next_magic<R: BitRange>(grng: &mut GameGenerator, range: R) -> u64 {
    let num_bits = grng.rng.random_range(range);
    let magicbb = grng.next_bitboard_with_n_bits_set(num_bits);
    magicbb.to_int()
}

pub fn validate_magic(
    attack_table: &mut [u64],
    magic: u64,
    baa: &[(u64, u64)],
    shift: usize,
) -> bool {
    for (blocker, attacker) in baa {
        let index = blocker.wrapping_mul(magic) >> shift;
        let entry = &mut attack_table[index as usize];
        if *entry == 0 {
            *entry = *attacker;
        } else if *entry != *attacker {
            return false;
        }
    }
    true
}
