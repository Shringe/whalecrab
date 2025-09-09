use std::cmp::Ordering;

use crate::{
    board::{Board, Color},
    movegen::moves::Move,
};

pub struct ScoredMove(pub Move, f32);

impl Ord for ScoredMove {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.partial_cmp(&other.1).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for ScoredMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScoredMove {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl Eq for ScoredMove {}

impl ScoredMove {
    pub fn from_moves(board: &Board, moves: Vec<Move>) -> Vec<Self> {
        let mut scored_moves = Vec::new();
        for m in moves {
            let new = m.make(board);
            let score = new.grade_position();
            let scored_move = ScoredMove(m, score);
            scored_moves.push(scored_move);
        }

        scored_moves
    }
}

impl Board {
    /// Grades the postion. For example, -1.0 means black is wining by a pawn's worth of value
    pub fn grade_position(&self) -> f32 {
        let mut score = 0.0;

        // Piece value
        let mut white_piece_value = 0.0;
        let mut black_piece_value = white_piece_value;
        for sq in self.occupied_bitboard() {
            match self
                .determine_color(sq)
                .expect("Expected piece on occupied_bitboard!")
            {
                Color::White => white_piece_value += self.determine_piece(sq).unwrap().value(),
                Color::Black => black_piece_value += self.determine_piece(sq).unwrap().value(),
            }
        }

        let piece_value = white_piece_value - black_piece_value;

        score += piece_value;
        score
    }

    /// Finds and returns the suggested move for the current players turn
    pub fn get_engine_move(&self) -> Option<ScoredMove> {
        let moves = self.generate_all_legal_moves();
        let mut scored_moves = ScoredMove::from_moves(self, moves);
        scored_moves.sort();
        match self.turn {
            Color::White => scored_moves.into_iter().next_back(),
            Color::Black => scored_moves.into_iter().next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::Square;

    #[test]
    fn engine_takes_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3";
        let board = Board::from_fen(starting).unwrap();
        let looking_for = Move::new(Square::C1, Square::G5, &board);
        let result = board.get_engine_move().expect("No moves found");
        assert_eq!(result.0, looking_for);
    }

    #[test]
    fn engine_saves_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 1 3";
        let board = Board::from_fen(starting).unwrap();
        let result = board.get_engine_move().expect("No moves found");
        let new = result.0.make(&board);
        assert_eq!(
            board.black_queen_bitboard.popcnt(),
            new.black_queen_bitboard.popcnt()
        );
    }
}
