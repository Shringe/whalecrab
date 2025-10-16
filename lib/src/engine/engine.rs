use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{
    bitboard::BitBoard,
    board::State,
    game::Game,
    movegen::{moves::Move, pieces::piece::Color},
};

impl Game {
    /// Grades the postion. For example, -1.0 means black is wining by a pawn's worth of value
    pub fn grade_position(&mut self) -> f32 {
        let mut hasher = DefaultHasher::new();
        self.position.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(pre) = self.transposition_table.get(&hash) {
            return *pre;
        }

        let mut score = 0.0;

        // Piece value
        for sq in self.occupied {
            let sqbb = BitBoard::from_square(sq);
            let (piece, color) = self.determine_piece(&sqbb).unwrap();

            match color {
                Color::White => {
                    score += piece.material_value();
                    score += piece.square_value(&sq, &color);
                }

                Color::Black => {
                    score -= piece.material_value();
                    score -= piece.square_value(&sq, &color);
                }
            }
        }

        // State
        match self.position.state {
            State::InProgress => {}
            State::Checkmate => {
                score = match self.position.turn {
                    Color::White => f32::NEG_INFINITY,
                    Color::Black => f32::INFINITY,
                }
            }
            State::Stalemate => score = 0.0,
            // TODO. Timing out should result in a win for the opponent if the opponent has
            // sufficent checkmating material
            State::Timeout => score = 0.0,
            State::Repetition => score = 0.0,
        }

        self.transposition_table.insert(hash, score);
        score
    }
    fn maxi(&mut self, mut alpha: f32, beta: f32, depth: u16) -> f32 {
        if depth == 0 {
            return self.grade_position();
        }

        let mut max = f32::NEG_INFINITY;
        for m in self.generate_all_legal_moves() {
            m.play(self);
            let score = self.mini(alpha, beta, depth - 1);
            m.unplay(self);
            if score > max {
                max = score;
                if score > alpha {
                    alpha = score;
                }
            }

            if score >= beta {
                break;
            }
        }

        max
    }

    fn mini(&mut self, alpha: f32, mut beta: f32, depth: u16) -> f32 {
        if depth == 0 {
            return self.grade_position();
        }

        let mut min = f32::INFINITY;
        for m in self.generate_all_legal_moves() {
            m.play(self);
            let score = self.maxi(alpha, beta, depth - 1);
            m.unplay(self);
            if score < min {
                min = score;
                if score < beta {
                    beta = score;
                }
            }

            if score <= alpha {
                break;
            }
        }

        min
    }

    pub fn get_engine_move_minimax(&mut self, depth: u16) -> Option<Move> {
        let moves = self.generate_all_legal_moves();
        let mut best_move = None;

        let alpha = f32::NEG_INFINITY;
        let beta = f32::INFINITY;

        match self.position.turn {
            Color::White => {
                let mut best_score = f32::NEG_INFINITY;
                for m in moves {
                    m.play(self);
                    let score = self.mini(alpha, beta, depth);
                    m.unplay(self);
                    if score > best_score {
                        best_score = score;
                        best_move = Some(m);
                    }
                }

                best_move
            }

            Color::Black => {
                let mut best_score = f32::INFINITY;
                for m in moves {
                    m.play(self);
                    let score = self.maxi(alpha, beta, depth);
                    m.unplay(self);
                    if score < best_score {
                        best_score = score;
                        best_move = Some(m);
                    }
                }

                best_move
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::{board::Board, square::Square};

    /// Used for determining cache hit/miss
    fn time_grading(game: &mut Game) -> (f32, Duration) {
        let start_time = Instant::now();
        let result = game.grade_position();
        let duration = start_time.elapsed();
        (result, duration)
    }

    #[test]
    fn minimax_engine_takes_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3";
        let mut game = Game::from_position(Board::from_fen(starting).unwrap());
        let looking_for = Move::new(Square::C1, Square::G5, &game.position);
        let result = game.get_engine_move_minimax(2).expect("No moves found");
        println!("State: {:?}", game.position.state);
        assert_eq!(result, looking_for);
    }

    #[test]
    fn minimax_engine_saves_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 1 3";
        let mut game = Game::from_position(Board::from_fen(starting).unwrap());
        let black_queens_before = game.position.black_queens.popcnt();
        let result = game.get_engine_move_minimax(2).expect("No moves found");
        game.play(&result);
        assert_eq!(black_queens_before, game.position.black_queens.popcnt());
    }

    #[test]
    fn transportation_table_cache_hits() {
        let mut game = Game::default();

        let (initial_result, initial_duration) = time_grading(&mut game);
        let min_speedup_factor = 1.5;

        for i in 1..100 {
            let (result, duration) = time_grading(&mut game);
            assert_eq!(initial_result, result);

            let speedup_factor = initial_duration.as_nanos() as f64 / duration.as_nanos() as f64;

            assert!(
                speedup_factor >= min_speedup_factor,
                "Grading #{} was only {:.2}x faster than initial, but should be at least {:.1}x faster. \
                Initial: {:?}, Current: {:?}",
                i, speedup_factor, min_speedup_factor, initial_duration, duration
            );
        }
    }
}
