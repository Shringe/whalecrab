use rand::RngExt;
use whalecrab_lib::{
    bitboard::{BitBoard, EMPTY},
    position::generator::GameGenerator,
    square::Square,
};

const NUM_ROOK_BITS: u8 = 12;
const NUM_ROOK_SUBSETS: usize = 1 << NUM_ROOK_BITS as usize;

fn find_rook_blockers_and_attackers(
    sq: Square,
    mask: BitBoard,
) -> (
    [BitBoard; NUM_ROOK_SUBSETS],
    [BitBoard; NUM_ROOK_SUBSETS],
    usize,
) {
    let mut blockers = [EMPTY; NUM_ROOK_SUBSETS];
    let mut attackers = [EMPTY; NUM_ROOK_SUBSETS];

    let mut subset = mask;
    for i in 0..NUM_ROOK_SUBSETS {
        blockers[i] = subset;
        attackers[i] = sq.rook_attacks_with_blockers(subset);
        if subset == EMPTY {
            return (blockers, attackers, i + 1);
        }
        subset = BitBoard::new(subset.to_int() - 1) & mask;
    }

    (blockers, attackers, 0)
}

fn next_magic(grng: &mut GameGenerator, num_bits: u8) -> u64 {
    let num_bits = grng.rng.random_range(1..=num_bits);
    let magicbb = grng.next_bitboard_with_n_bits_set(num_bits);
    magicbb.to_int()
}

fn test_magic(magic: u64, blockers: &[BitBoard], attackers: &[BitBoard], shift: usize) -> bool {
    let mut table = [EMPTY; NUM_ROOK_SUBSETS];
    for (blocker, attacker) in blockers.iter().zip(attackers.iter()) {
        let idx = blocker.to_int().wrapping_mul(magic) >> shift;
        let entry = &mut table[idx as usize];

        if *entry == EMPTY {
            *entry = *attacker;
        } else if *entry != *attacker {
            return false;
        }
    }
    true
}

fn main() {
    let mut grng = GameGenerator::unseeded();

    for sq in Square::ALL_SQUARES {
        let mask = sq.rook_mask();
        if mask == EMPTY {
            continue;
        }

        let num_bits = mask.popcnt() as u8;
        let shift = 64 - num_bits as usize;

        let (blockers, attackers, len) = find_rook_blockers_and_attackers(sq, mask);

        loop {
            let magic = next_magic(&mut grng, num_bits);
            let valid = test_magic(magic, &blockers[..len], &attackers[..len], shift);

            if valid {
                eprintln!("Found magic with {num_bits} bits: {magic}");
                break;
            }
        }
    }
}
