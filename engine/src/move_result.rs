use whalecrab_lib::movegen::moves::Move;

use crate::score::Score;

#[derive(Debug, Default)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: Score,
    pub depth: u16,
    pub nodes: u64,
}
