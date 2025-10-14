use std::{
    cmp::Ordering,
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
    bitboard::BitBoard,
    game::Game,
    movegen::{moves::Move, pieces::piece::Color},
};

#[derive(Clone)]
pub struct ScoredMove(pub Move, f32);

impl fmt::Display for ScoredMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {}, {:?}, {}",
            self.0.from, self.0.to, self.0.variant, self.1
        )
    }
}

impl fmt::Debug for ScoredMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

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
    pub fn from_moves(game: &mut Game, moves: Vec<Move>) -> Vec<Self> {
        let mut scored_moves = Vec::new();
        for m in moves {
            let mut new = game.clone();
            new.play(&m);
            let score = new.grade_position();
            let scored_move = ScoredMove(m, score);
            scored_moves.push(scored_move);
        }

        scored_moves
    }
}

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

        self.transposition_table.insert(hash, score);
        score
    }

    /// Finds and returns the suggested move for the current players turn
    pub fn find_best_move(&mut self) -> Option<ScoredMove> {
        let moves = self.generate_all_legal_moves();
        let mut scored_moves = ScoredMove::from_moves(self, moves);
        scored_moves.sort();
        match self.position.turn {
            Color::White => scored_moves.into_iter().next_back(),
            Color::Black => scored_moves.into_iter().next(),
        }
    }

    /// Recusively searches through the specified depth (in half moves) to find the best move
    pub fn get_engine_move(
        &mut self,
        depth: u16,
        initial: Option<ScoredMove>,
    ) -> Option<ScoredMove> {
        if depth == 0 {
            let best = self.find_best_move();
            return best;
        }

        let moves = self.generate_all_legal_moves();
        let scored_moves = ScoredMove::from_moves(self, moves);
        let mut best_initial = None;
        let mut best_final = None;
        for m in scored_moves {
            let mut board = self.clone();
            board.play(&m.0);

            let initial = if initial.is_some() {
                initial.clone()
            } else {
                Some(m)
            };

            if let Some(m) = board.get_engine_move(depth - 1, initial.clone()) {
                if let Some(b) = &best_final {
                    if &m > b {
                        best_initial = initial;
                        best_final = Some(m);
                    }
                } else {
                    best_initial = initial;
                    best_final = Some(m)
                }
            }
        }

        best_initial
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
    fn old_engine_takes_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3";
        let mut game = Game::from_position(Board::from_fen(starting).unwrap());
        let looking_for = Move::new(Square::C1, Square::G5, &game.position);
        let result = game.find_best_move().expect("No moves found");
        assert_eq!(result.0, looking_for);
    }

    #[test]
    fn old_engine_saves_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 1 3";
        let mut game = Game::from_position(Board::from_fen(starting).unwrap());
        let black_queens_before = game.position.black_queens.popcnt();
        let result = game.find_best_move().expect("No moves found");
        game.play(&result.0);
        assert_eq!(black_queens_before, game.position.black_queens.popcnt());
    }

    #[test]
    fn minimax_engine_takes_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3";
        let mut game = Game::from_position(Board::from_fen(starting).unwrap());
        let looking_for = Move::new(Square::C1, Square::G5, &game.position);
        let result = game.get_engine_move_minimax(2).expect("No moves found");
        let moves = game.generate_all_legal_moves();
        let mut scored = ScoredMove::from_moves(&mut game, moves);
        scored.sort();
        println!("Available moves: {:#?}", scored);
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
