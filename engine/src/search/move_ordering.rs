use whalecrab_lib::movegen::{moves::Move, pieces::piece::PieceType};

use crate::{
    piece_eval::material_value, score::Score, transposition_table::TranspositionTableEntry,
};

/// Scores a move. This can be used for move ordering
fn score_move(m: &Move, best: Option<&Move>) -> Score {
    if Some(m) == best {
        return Score::MIN;
    }

    match m {
        Move::Promotion {
            piece,
            capture: Some(capture),
            ..
        } => Score::new(-5000) - material_value(*piece) - material_value(*capture),
        Move::Promotion {
            piece,
            capture: None,
            ..
        } => Score::new(-5000) - material_value(*piece),
        Move::CaptureEnPassant { .. } => Score::new(-2000) - material_value(PieceType::Pawn),
        Move::Normal {
            capture: Some(capture),
            ..
        } => Score::new(-2000) - material_value(*capture),
        Move::Castle { .. } => Score::new(-500),
        _ => Score::new(0),
    }
}

/// Orders the moves for better minimax pruning
pub fn order_moves(mut moves: Vec<Move>, existing: &Option<&TranspositionTableEntry>) -> Vec<Move> {
    let best_move = existing.and_then(|e| e.best_move.as_ref());

    moves.sort_unstable_by_key(|m| score_move(m, best_move));

    moves
}

#[cfg(test)]
mod tests {
    use crate::engine::Engine;

    use super::*;

    #[test]
    fn sort_moves_keeps_all_moves() {
        let mut engine = Engine::default();
        let moves = engine.game.legal_moves();
        let sorted = order_moves(moves.clone(), &None);
        for sortedm in &sorted {
            assert!(moves.contains(sortedm));
        }
        assert_eq!(sorted.len(), moves.len());
    }
}
