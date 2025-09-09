use crate::board::{Board, Color};
use rand::Rng;

impl Board {
    /// Grades the postion. For example, -1.0 means black is wining by a pawn's worth of value
    /// Currently just produces a random number
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

        let piece_value = match self.turn {
            Color::White => white_piece_value - black_piece_value,
            Color::Black => black_piece_value - white_piece_value,
        };

        score += piece_value;
        score
    }

    /// Finds the top engine move for the current position and makes it on a new board
    pub fn make_engine_move(&self) -> Board {
        let moves = self.generate_all_legal_moves();
        let mut best_position = moves.get(0).expect("No moves to grade!").make(self);
        let mut best_score = best_position.grade_position();
        for m in moves {
            let board = m.make(self);
            let score = board.grade_position();
            if score > best_score {
                best_score = score;
                best_position = board;
            }
        }

        best_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_takes_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3";
        let looking_for = "rnb1kbnr/pppp1ppp/8/4p1B1/3PP3/8/PPP2PPP/RN1QKBNR b KQkq - 0 3";
        let board = Board::from_fen(starting).unwrap();
        let new = board.make_engine_move();
        assert_eq!(new.to_fen(), Board::from_fen(looking_for).unwrap().to_fen());
    }
}
