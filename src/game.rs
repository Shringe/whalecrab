use std::collections::HashMap;

use crate::{bitboard::BitBoard, board::Board};

#[derive(Default)]
pub struct Game {
    pub position: Board,
    pub transposition_table: HashMap<u64, f32>,
    pub white_attack_bitboard: BitBoard,
    pub black_attack_bitboard: BitBoard,
    pub white_attack_ray_bitboard: BitBoard,
    pub black_attack_ray_bitboard: BitBoard,
    pub white_num_checks: u8,
    pub black_num_checks: u8,
}
