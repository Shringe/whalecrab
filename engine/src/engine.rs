use std::{collections::HashMap, time::Duration};

use crate::{
    piece_eval::{material_value, square_value},
    platform_timer,
    score::Score,
    timers::{MoveTimer, infinite::Infinite},
};
use whalecrab_lib::{
    bitboard::BitBoard,
    file::File,
    game::{Game, State},
    movegen::{moves::Move, pieces::piece::PieceColor},
    square::Square,
};

/// Plays a move, gets the score from the given method, and then unplays the move and returns that
/// score. Also does expensive validity checks in debug builds.
macro_rules! search_move {
    ($self:expr, $move:expr, $method:ident($($args:expr),*)) => {{
        #[cfg(debug_assertions)]
        let before = $self.game.clone();

        $self.game.play(&$move);

        #[cfg(debug_assertions)]
        let during = $self.game.clone();

        $self.nodes_searched += 1;
        let score = $self.$method($($args),*);
        $self.game.unplay(&$move);

        #[cfg(debug_assertions)]
        assert_eq!(
            $self.game, before,
            "State changed after playing and unplaying {}\n  Before: {:?}\n  During: {:?}\n   After: {:?}\n",
            $move, before, during, $self.game
        );

        score
    }};
}

/// Orders the moves for better minimax pruning
/// TODO: Figure out why the reduction in nodes searched is minimal. The outcome of the game is
/// also being changed sometimes
fn order_moves(mut moves: Vec<Move>) -> Vec<Move> {
    // return moves;

    moves.sort_unstable_by_key(|m| match m {
        Move::Promotion { .. } => 0,
        _ if m.is_capture() => 1,
        Move::Castle { .. } => 2,
        _ => 3,
    });

    moves
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Engine {
    /// Use self.with_new_game(game) instead of self.game = game if you want to replace this value
    pub game: Game,
    transposition_table: HashMap<u64, Score>,
    pub nodes_searched: u64,
}

impl Engine {
    pub fn from_game(game: Game) -> Engine {
        Engine {
            game,
            transposition_table: HashMap::new(),
            nodes_searched: 0,
        }
    }

    /// Creates a position from fen and wraps the engine around it
    pub fn from_fen(fen: &str) -> Option<Engine> {
        Some(Engine::from_game(Game::from_fen(fen)?))
    }

    /// Resets any temporary engine values or caches and switches over to analyzing the new game.
    /// This should be used over replacing self.game manually
    pub fn with_new_game(&mut self, game: Game) {
        self.game = game
    }

    /// Clears caches that do not need bo be reset each game. This should only be called for
    /// testing and benchmarking purposes
    pub fn clear_persistant_cache(&mut self) {
        self.transposition_table.clear();
    }

    /// Score material based on its value and position on the board
    fn score_material(&self) -> Score {
        let mut score = Score::default();

        for sq in self.game.occupied {
            let sqbb = BitBoard::from_square(sq);
            let (piece, color) = self.game.determine_piece(&sqbb).unwrap();

            match color {
                PieceColor::White => {
                    score += material_value(&piece);
                    score += square_value(&piece, &sq, &color);
                }

                PieceColor::Black => {
                    score -= material_value(&piece);
                    score -= square_value(&piece, &sq, &color);
                }
            }
        }

        score
    }

    /// Scores king safety. Primarily based on whether the king has friendly pawns next to him.
    fn score_king_safety(&self) -> Score {
        let mut score = Score::default();

        let calculate_pawn_area = |king: &Square| {
            let file = king.get_file();
            let mut pawn_area = file.mask();
            if file > File::A {
                pawn_area |= file.left().mask();
            }
            if file < File::H {
                pawn_area |= file.right().mask();
            }
            pawn_area
        };

        let white_king = self.game.white_kings.to_square();
        let white_pawn_area = calculate_pawn_area(&white_king);
        score += Score::new(
            ((white_pawn_area & self.game.white_pawns).popcnt() * 15)
                .try_into()
                .unwrap(),
        );

        let black_king = self.game.black_kings.to_square();
        let black_pawn_area = calculate_pawn_area(&black_king);
        score -= Score::new(
            ((black_pawn_area & self.game.black_pawns).popcnt() * 15)
                .try_into()
                .unwrap(),
        );

        score
    }

    /// Scores the position castling rights
    fn score_castling_rights(&self) -> Score {
        let mut score = Score::default();
        let value = 2;

        if self.game.castling_rights.white_queenside {
            score += Score::new(value);
        }

        if self.game.castling_rights.white_kingside {
            score += Score::new(value);
        }

        if self.game.castling_rights.black_queenside {
            score -= Score::new(value);
        }

        if self.game.castling_rights.black_kingside {
            score -= Score::new(value);
        }

        score
    }

    /// Scores both attackers and defenders
    fn score_attackers(&self) -> Score {
        let mut score = Score::default();

        score += Score::new(
            ((self.game.white_attacks & self.game.occupied).popcnt() * 10)
                .try_into()
                .unwrap(),
        );
        score -= Score::new(
            ((self.game.black_attacks & self.game.occupied).popcnt() * 10)
                .try_into()
                .unwrap(),
        );

        score
    }

    /// Grades the postion. For example, -1.0 means black is wining by a pawn's worth of value
    pub fn grade_position(&mut self) -> Score {
        if let Some(pre) = self.transposition_table.get(&self.game.hash) {
            return *pre;
        }

        macro_rules! end {
            ($score: expr) => {{
                self.transposition_table.insert(self.game.hash, $score);
                return $score;
            }};
        }

        // State
        match self.game.state {
            State::InProgress => {}
            State::Checkmate => {
                end!(match self.game.turn {
                    PieceColor::White => Score::MIN,
                    PieceColor::Black => Score::MAX,
                })
            }
            State::Stalemate => end!(Score::default()),
            // TODO. Timing out should result in a win for the opponent if the opponent has
            // sufficent checkmating material
            State::Timeout => end!(Score::default()),
            State::Repetition => end!(Score::default()),
        }

        let mut score = Score::default();

        score += self.score_material();
        score += self.score_attackers();
        score += self.score_king_safety();
        score += self.score_castling_rights();

        end!(score)
    }

    fn maxi<T: MoveTimer>(
        &mut self,
        mut alpha: Score,
        beta: Score,
        depth: u16,
        timer: &T,
    ) -> Score {
        if depth == 0 || timer.over() {
            return self.grade_position();
        }

        let mut max = Score::MIN;
        for m in order_moves(self.game.legal_moves()) {
            let score = search_move!(self, m, mini(alpha, beta, depth - 1, timer));
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

    fn mini<T: MoveTimer>(
        &mut self,
        alpha: Score,
        mut beta: Score,
        depth: u16,
        timer: &T,
    ) -> Score {
        if depth == 0 || timer.over() {
            return self.grade_position();
        }

        let mut min = Score::MAX;
        for m in order_moves(self.game.legal_moves()) {
            let score = search_move!(self, m, maxi(alpha, beta, depth - 1, timer));
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

    /// Contiunes searching until the depth is reached
    pub fn minimax(&mut self, depth: u16) -> Option<Move> {
        self.minimax_with_duration(depth, &Infinite).0
    }

    /// Continues searching until either the depth or duration is reached
    pub fn minimax_with_duration<T: MoveTimer>(
        &mut self,
        depth: u16,
        timer: &T,
    ) -> (Option<Move>, bool) {
        let moves = order_moves(self.game.legal_moves());
        let mut best_move = None;

        let mut alpha = Score::MIN;
        let mut beta = Score::MAX;

        macro_rules! search_loop {
            ($best_score:expr, $cmp:tt, $search:ident, $prune:expr) => {{
                let mut best_score = $best_score;
                for m in moves {
                    if timer.over() {
                        return (best_move, true);
                    }

                    let score = search_move!(self, m, $search(alpha, beta, depth, timer));
                    if score $cmp best_score {
                        best_score = score;
                        best_move = Some(m);
                        if score $cmp $prune {
                            $prune = score;
                        }
                    }
                }
                (best_move, false)
            }};
        }

        match self.game.turn {
            PieceColor::White => search_loop!(Score::MIN, >, mini, alpha),
            PieceColor::Black => search_loop!(Score::MAX, <, maxi, beta),
        }
    }

    /// The engine will continue searching deeper and deeper depths until the duration has passed,
    /// at which point it will return the best move found so far.
    pub fn iterative_deepening(&mut self, duration: &Duration) -> Option<Move> {
        let timer = platform_timer!(*duration);
        self.iterative_deepening_with_timer(&timer)
    }

    pub fn iterative_deepening_with_timer<T: MoveTimer>(&mut self, timer: &T) -> Option<Move> {
        let mut depth = 0;
        let mut best_move_so_far = None;

        loop {
            let (best_move, ran_out_of_time) = self.minimax_with_duration(depth, timer);

            if ran_out_of_time {
                return best_move_so_far;
            }

            best_move_so_far = best_move;
            depth += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::timers::elapsed::Elapsed;

    use super::*;
    use whalecrab_lib::{movegen::pieces::piece::PieceType, square::Square};

    /// Used for determining cache hit/miss
    fn time_grading(engine: &mut Engine) -> (Score, Duration) {
        let start_time = Instant::now();
        let result = engine.grade_position();
        let duration = start_time.elapsed();
        (result, duration)
    }

    #[track_caller]
    fn assert_iterative_deepening_timing<T: MoveTimer, M: FnOnce(Duration) -> T>(make_timer: M) {
        let mut engine = Engine::default();

        let duration = Duration::from_millis(1000);
        let min = Duration::from_micros((duration.as_micros() as f64 * 0.98) as u64);
        let max = Duration::from_micros((duration.as_micros() as f64 * 1.02) as u64);

        let timer = make_timer(duration);
        let now = Instant::now();
        assert!(!timer.over());
        let _ = engine.iterative_deepening_with_timer(&timer);
        assert!(timer.over());
        let elapsed = now.elapsed();

        assert!(
            elapsed > min,
            "iterative_deepening for {:?} should have completed after {:?}, but took {:?}",
            duration,
            min,
            elapsed
        );

        assert!(
            elapsed < max,
            "iterative_deepening for {:?} should have completed within {:?}, but took {:?}",
            duration,
            max,
            elapsed
        );
    }

    #[test]
    fn iterative_deepening_should_take_the_right_amount_of_time_on_elapsed() {
        assert_iterative_deepening_timing(Elapsed::now);
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn iterative_deepening_should_take_the_right_amount_of_time_on_rdtsc() {
        assert_iterative_deepening_timing(crate::timers::rdtsc::Rdtsc::now);
    }

    #[test]
    fn iterative_deepening_should_take_the_right_amount_of_time_on_platform() {
        assert_iterative_deepening_timing(|duration| platform_timer!(duration));
    }

    #[test]
    fn grading_should_not_mutate_position() {
        let mut engine = Engine::default();
        let before = engine.game.clone();
        let grade = engine.grade_position();
        let after = engine.game;
        println!("Score: {:?}", grade);
        assert_eq!(before, after);
    }

    #[test]
    fn iterative_deepening_finds_a_move() {
        let mut engine = Engine::default();
        let duration = Duration::from_millis(200);
        let best_move = engine.iterative_deepening(&duration);
        assert!(best_move.is_some());
    }

    #[test]
    fn starting_evaluation_is_balanced() {
        let mut engine = Engine::default();
        println!("{:?}", engine.game);
        let grade = engine.grade_position();
        println!("{:?}", engine.game);
        assert_eq!(grade, Score::default());
    }

    #[test]
    fn grade_position_is_deterministic() {
        let mut engine = Engine::default();
        let mut last_score = engine.grade_position();
        let mut last = engine.game.clone();
        for _ in 1..20 {
            let score = engine.grade_position();
            let game = engine.game.clone();
            assert_eq!(score, last_score);
            assert_eq!(game, last);
            last = game;
            last_score = score;
        }
    }

    #[test]
    fn minimax_engine_takes_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3";
        let mut engine = Engine::from_fen(starting).unwrap();
        let looking_for = Move::infer(Square::C1, Square::G5, &engine.game);
        let result = engine.minimax(2).expect("No moves found");
        println!("State: {:?}", engine.game.state);
        assert_eq!(result, looking_for);
    }

    #[test]
    fn minimax_engine_saves_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 1 3";
        let mut engine = Engine::from_fen(starting).unwrap();
        let black_queens_before = engine.game.black_queens.popcnt();
        let result = engine.minimax(2).expect("No moves found");
        engine.game.play(&result);
        assert_eq!(black_queens_before, engine.game.black_queens.popcnt());
    }

    #[test]
    fn black_always_takes_king() {
        let fen = "k6r/pp4r1/8/pp6/Qp6/pp6/7K/8 w - - 0 1";
        let mut engine = Engine::from_fen(fen).unwrap();
        let white_moves = engine.game.legal_moves();
        for m in white_moves {
            engine.game.play(&m);
            let result = engine.minimax(0).unwrap();
            assert!(
                matches!(
                    result,
                    Move::Normal {
                        capture: Some(PieceType::King),
                        ..
                    }
                ),
                "Expected black to capture the king, got {:?}",
                result
            );
            engine.game.unplay(&m);
        }
    }

    #[test]
    fn white_always_checkmates() {
        let fen = "7k/8/8/8/8/8/5R2/K5R1 b - - 0 1";
        let mut engine = Engine::from_fen(fen).unwrap();
        let black_moves = engine.game.legal_moves();
        for m in black_moves {
            engine.game.play(&m);
            let looking_for = Move::infer(Square::F2, Square::H2, &engine.game);
            let result = engine.minimax(1).unwrap();
            assert_eq!(result, looking_for);
            engine.game.unplay(&m);
        }
    }

    #[test]
    fn transportation_table_cache_hits() {
        let mut engine = Engine::default();

        let (initial_result, initial_duration) = time_grading(&mut engine);
        let min_speedup_factor = 1.2;

        for i in 1..100 {
            let (result, duration) = time_grading(&mut engine);
            assert_eq!(initial_result, result);

            let speedup_factor = initial_duration.as_nanos() as f64 / duration.as_nanos() as f64;

            assert!(
                speedup_factor >= min_speedup_factor,
                "Grading #{} was only {:.2}x faster than initial, but should be at least {:.1}x faster. \
                Initial: {:?}, Current: {:?}",
                i,
                speedup_factor,
                min_speedup_factor,
                initial_duration,
                duration
            );
        }
    }

    #[test]
    fn engine_moves_immutably() {
        let fen = "rnbqkbnr/pp1ppppp/2p5/8/4PP2/8/PPPP2PP/RNBQKBNR b KQkq - 0 2";
        let mut engine = Engine::from_fen(fen).unwrap();
        let before = engine.game.clone();
        let _ = engine.game.legal_moves();
        let _ = engine.minimax(2);
        assert_eq!(before, engine.game);
    }

    #[test]
    fn sort_moves_keeps_all_moves() {
        let mut engine = Engine::default();
        let moves = engine.game.legal_moves();
        let sorted = order_moves(moves.clone());
        for sortedm in &sorted {
            assert!(moves.contains(sortedm));
        }
        assert_eq!(sorted.len(), moves.len());
    }

    #[test]
    fn engine_should_not_mutate_position() {
        let fen = "r1k2b1r/1p4p1/p1p4P/4B3/2p5/3P3P/NP2P1B1/2K2R2 w - - 0 29";
        let mut engine = Engine::from_fen(fen).unwrap();
        let before = engine.game.clone();
        let _ = engine.minimax(3);
        let after = engine.game;
        assert_eq!(after, before);
    }

    #[test]
    fn should_have_moves_fen() {
        let fen = "rnbqkbnr/pp1ppppp/2p5/8/4PP2/8/PPPP2PP/RNBQKBNR b KQkq f3 0 2";
        let mut engine = Engine::from_fen(fen).unwrap();
        let moves = engine.game.legal_moves();
        let engine_move = engine.minimax(2);
        assert!(!moves.is_empty());
        assert!(engine_move.is_some())
    }

    #[test]
    fn should_have_moves() {
        let mut engine = Engine::default();
        for (from, to) in [
            (Square::E2, Square::E4),
            (Square::C7, Square::C6),
            (Square::F2, Square::F4),
        ] {
            let m = Move::infer(from, to, &engine.game);
            engine.game.play(&m);
            let moves = engine.game.legal_moves();
            let engine_move = engine.minimax(2);
            assert_eq!(engine.game.state, State::InProgress);
            assert!(!moves.is_empty());
            assert!(engine_move.is_some())
        }
    }
}
