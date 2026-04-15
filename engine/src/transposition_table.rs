use whalecrab_lib::movegen::moves::Move;

use crate::score::Score;

#[derive(Default, Clone, Debug, PartialEq)]
pub(crate) struct TranspositionTableEntry {
    pub(crate) best_move: Option<Move>,
    pub(crate) depth: u16,
    pub(crate) score: Score,
}
