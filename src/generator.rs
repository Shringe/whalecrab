use crate::board::Board;

struct MoveGenerator {
    board: Board,
}

impl MoveGenerator {
    pub fn white_pawn(&self, position: u64) -> u64 {
        (position - 8) & (position - 16)
    }
}
