use std::fmt;

use rand::{RngExt, distr::uniform::SampleRange};
use whalecrab_lib::{
    bitboard::{BitBoard, EMPTY},
    position::generator::GameGenerator,
    square::Square,
};

#[derive(Default, Clone, Copy)]
struct Magic {
    mask: BitBoard,
    magic: u64,
}

impl fmt::Debug for Magic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fancy = f.alternate();
        let mut s = f.debug_struct("Magic");
        s.field("magic", &self.magic);
        s.field("mask", &self.mask.to_int());
        if fancy {
            s.field("magic.popcnt()", &self.magic.count_ones());
            s.field("mask.popcnt()", &self.mask.popcnt());
        }
        s.finish()
    }
}

const NUM_ROOK_BITS: u8 = 12;
const NUM_ROOK_SUBSETS: usize = 1 << NUM_ROOK_BITS as usize;
const ROOK_SHIFT: usize = 64 - NUM_ROOK_BITS as usize;

fn generate_rook_blockers_and_attackers(
    sq: Square,
    mask: BitBoard,
) -> ([(BitBoard, BitBoard); NUM_ROOK_SUBSETS], usize) {
    let mut baa = [(EMPTY, EMPTY); NUM_ROOK_SUBSETS];

    let mut subset = mask;
    for i in 0..NUM_ROOK_SUBSETS {
        baa[i] = (subset, sq.rook_attacks_with_blockers(subset));
        if subset == EMPTY {
            return (baa, i + 1);
        }
        subset = BitBoard::new(subset.to_int() - 1) & mask;
    }

    panic!();
}

fn next_magic<R: SampleRange<u8>>(grng: &mut GameGenerator, range: R) -> u64 {
    let num_bits = grng.rng.random_range(range);
    let magicbb = grng.next_bitboard_with_n_bits_set(num_bits);
    magicbb.to_int()
}

fn validate_magic(
    table: &mut [BitBoard],
    magic: u64,
    baa: &[(BitBoard, BitBoard)],
    shift: usize,
) -> bool {
    for (blocker, attacker) in baa {
        let index = blocker.to_int().wrapping_mul(magic) >> shift;
        let entry = &mut table[index as usize];
        if *entry == EMPTY {
            *entry = *attacker;
        } else if *entry != *attacker {
            return false;
        }
    }
    true
}

fn find_rook_magics<R: SampleRange<u8> + Clone>(
    grng: &mut GameGenerator,
    range: R,
) -> ([[BitBoard; NUM_ROOK_SUBSETS]; 64], [Magic; 64]) {
    let mut magics = [Magic::default(); 64];
    let mut attacks = [[(EMPTY); NUM_ROOK_SUBSETS]; 64];

    for sq in Square::ALL_SQUARES {
        let mask = sq.rook_mask();

        let (baa, len) = generate_rook_blockers_and_attackers(sq, mask);

        loop {
            let mut table = [EMPTY; NUM_ROOK_SUBSETS];
            let magic = next_magic(grng, range.clone());
            let valid = validate_magic(&mut table, magic, &baa[..len], ROOK_SHIFT);

            if valid {
                magics[sq.index()] = Magic { mask, magic };
                attacks[sq.index()] = table;
                break;
            }
        }
    }

    (attacks, magics)
}

fn rook_attacks(
    sq: Square,
    occupied: BitBoard,
    attacks: &[[BitBoard; NUM_ROOK_SUBSETS]; 64],
    magics: &[Magic; 64],
) -> BitBoard {
    let magic = magics[sq.index()];
    let key = ((occupied & magic.mask) * magic.magic) >> (ROOK_SHIFT as u64);
    attacks[sq.index()][key.to_int() as usize]
}

fn main() {
    let mut grng = GameGenerator::unseeded();

    let (attack_table, magics) = find_rook_magics(&mut grng, 12..=12);
    println!("{:#?}\n\n", &magics);

    let sq = Square::E8;
    let expected = sq.rook_attacks_with_blockers(EMPTY);
    let actual = rook_attacks(sq, EMPTY, &attack_table, &magics);
    assert_eq!(actual, expected);
    eprintln!("{}\n\n{}", actual, expected);
}
