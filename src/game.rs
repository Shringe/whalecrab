use std::collections::HashMap;

use crate::{
    bitboard::BitBoard,
    board::{color_field_getters, Board},
};

#[derive(Default)]
pub struct Game {
    pub position: Board,
    pub transposition_table: HashMap<u64, f32>,
    pub white_attacks: BitBoard,
    pub black_attacks: BitBoard,
    pub white_check_rays: BitBoard,
    pub black_check_rays: BitBoard,
    pub white_num_checks: u8,
    pub black_num_checks: u8,
}

impl Game {
    color_field_getters!(attacks, BitBoard);
    color_field_getters!(check_rays, BitBoard);
    color_field_getters!(num_checks, u8);
}
