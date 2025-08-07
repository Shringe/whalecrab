use crate::{
    board::Board,
    movegen::moves::{get_targets, Move},
    square::Square,
};

pub trait Piece {
    /// Generates psuedo legal moves not considering king safety.
    fn psuedo_legal_moves(&self, board: &Board) -> Vec<Move>;

    /// Generates psuedo legal targets. Useful for highlighting squares in the TUI.
    fn psuedo_legal_targets(&self, board: &Board) -> Vec<Square> {
        let moves = self.psuedo_legal_moves(board);
        get_targets(moves)
    }
}
