use crate::{board::Board, movegen::moves::Move, square::Square};

pub trait Piece {
    /// Generates psuedo legal moves not considering king safety.
    fn psuedo_legal_moves(&self, board: &Board) -> Vec<Move>;

    /// Generates psuedo legal targets. Useful for highlighting squares in the TUI.
    fn psuedo_legal_targets(&self, board: &Board) -> Vec<Square> {
        let moves = self.psuedo_legal_moves(board);
        let mut targets = Vec::new();
        for m in moves {
            targets.push(m.to)
        }
        targets
    }
}
