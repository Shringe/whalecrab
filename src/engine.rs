use crate::board::Board;
use rand::Rng;

impl Board {
    /// Grades the postion. For example, -1.0 means black is wining by a pawn's worth of value
    /// Currently just produces a random number
    fn grade_position(&self) -> f32 {
        let mut rng = rand::rng();
        let range = 0.3;
        rng.random_range((range * -1.0)..range)
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
