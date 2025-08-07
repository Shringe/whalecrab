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

    /// Generates legal moves considering king safety.
    fn legal_moves(&self, board: &Board) -> Vec<Move> {
        let psuedo_legal = self.psuedo_legal_moves(&board);
        let mut legal = Vec::new();

        for m in psuedo_legal {
            let new = m.make(&board);
            if !new.is_king_in_check() {
                legal.push(m);
            }
        }

        legal
    }

    /// Generates legal targets. Useful for highlighting squares in the TUI.
    fn legal_targets(&self, board: &Board) -> Vec<Square> {
        let moves = self.legal_moves(board);
        get_targets(moves)
    }
}
