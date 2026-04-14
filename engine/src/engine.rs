use std::{collections::HashMap, time::Duration};

use crate::{
    move_result::{SearchInfo, SearchResult},
    piece_eval::{material_value, square_value},
    platform_timer,
    score::Score,
    timers::{MoveTimer, infinite::Infinite},
    transposition_table::TranspositionTableEntry,
};
use whalecrab_lib::{
    file::File,
    movegen::{
        moves::Move,
        pieces::piece::{PieceColor, PieceType},
    },
    position::game::{Game, State},
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

        let score = $self.$method($($args),*);
        $self.game.unplay($move);

        #[cfg(debug_assertions)]
        assert_eq!(
            $self.game, before,
            "State changed after playing and unplaying {}\n  Before: {:?}\n  During: {:?}\n   After: {:?}\n",
            $move, before, during, $self.game
        );

        score
    }};
}

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
fn order_moves(mut moves: Vec<Move>, existing: &Option<&TranspositionTableEntry>) -> Vec<Move> {
    let best_move = existing.and_then(|e| e.best_move.as_ref());

    moves.sort_unstable_by_key(|m| score_move(m, best_move));

    moves
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Engine {
    /// Use self.with_new_game(game) instead of self.game = game if you want to replace this value
    pub game: Game,
    transposition_table: HashMap<u64, TranspositionTableEntry>,
}

impl Engine {
    pub fn from_game(game: Game) -> Engine {
        Engine {
            game,
            transposition_table: HashMap::new(),
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

    fn score_white_material_positive(&self) -> Score {
        let mut score = Score::default();

        score += material_value(PieceType::Pawn) * self.game.white_pawns.popcnt() as i32;
        score += material_value(PieceType::Knight) * self.game.white_knights.popcnt() as i32;
        score += material_value(PieceType::Bishop) * self.game.white_bishops.popcnt() as i32;
        score += material_value(PieceType::Rook) * self.game.white_rooks.popcnt() as i32;
        score += material_value(PieceType::Queen) * self.game.white_queens.popcnt() as i32;
        score += material_value(PieceType::King) * self.game.white_kings.popcnt() as i32;

        score
    }

    fn score_black_material_positive(&self) -> Score {
        let mut score = Score::default();

        score += material_value(PieceType::Pawn) * self.game.black_pawns.popcnt() as i32;
        score += material_value(PieceType::Knight) * self.game.black_knights.popcnt() as i32;
        score += material_value(PieceType::Bishop) * self.game.black_bishops.popcnt() as i32;
        score += material_value(PieceType::Rook) * self.game.black_rooks.popcnt() as i32;
        score += material_value(PieceType::Queen) * self.game.black_queens.popcnt() as i32;
        score += material_value(PieceType::King) * self.game.black_kings.popcnt() as i32;

        score
    }

    fn midgame_to_lategame_ratio(&self, material_score: Score) -> f64 {
        let max_material = material_value(PieceType::Queen) * 1
            + material_value(PieceType::Rook) * 2
            + material_value(PieceType::Bishop) * 2
            + material_value(PieceType::Knight) * 2
            + material_value(PieceType::Pawn) * 8;

        let material_ratio =
            material_score.min(max_material).to_int() as f64 / max_material.to_int() as f64;
        let clock_penalty = (self.game.full_move_clock as f64 / 400.0).min(0.2);

        (material_ratio - clock_penalty).clamp(0.0, 1.0)
    }

    /// Score material based on its value and position on the board
    fn score_pieces(&self) -> Score {
        let white_material_score = self.score_white_material_positive();
        let black_material_score = self.score_black_material_positive();
        let ratio = self.midgame_to_lategame_ratio(white_material_score + black_material_score);
        let mut score = white_material_score - black_material_score;

        for sq in self.game.occupied {
            let (piece, color) = self.game.piece_lookup(sq).unwrap();
            match color {
                PieceColor::White => {
                    score += square_value(piece, sq, color, ratio);
                }
                PieceColor::Black => {
                    score -= square_value(piece, sq, color, ratio);
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

        if self.game.castling_rights.white_queenside() {
            score += Score::new(value);
        }

        if self.game.castling_rights.white_kingside() {
            score += Score::new(value);
        }

        if self.game.castling_rights.black_queenside() {
            score -= Score::new(value);
        }

        if self.game.castling_rights.black_kingside() {
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
        // if let Some(pre) = self.transposition_table.get(&self.game.hash) {
        //     return *pre;
        // }

        macro_rules! end {
            ($score: expr) => {{
                // self.transposition_table.insert(self.game.hash, $score);
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

        score += self.score_pieces();
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
    ) -> SearchInfo {
        if depth == 0 || timer.over() {
            return SearchInfo {
                score: self.grade_position(),
                depth,
                nodes: 1,
            };
        }

        let existing = self.transposition_table.get(&self.game.hash);
        let better_than_existing = existing.is_none_or(|e| depth > e.depth);

        let mut result = SearchResult::new(Score::MIN, depth);

        for m in order_moves(self.game.legal_moves(), &existing) {
            let node = search_move!(self, &m, mini(alpha, beta, depth - 1, timer));
            result += &node;

            if node.score > result.info.score {
                result.info.score = node.score;
                result.best_move = Some(m);
                if node.score > alpha {
                    alpha = node.score;
                }
            }

            if node.score >= beta {
                break;
            }
        }

        if better_than_existing {
            let entry = TranspositionTableEntry {
                best_move: result.best_move,
                depth,
            };
            self.transposition_table.insert(self.game.hash, entry);
        }

        result.info
    }

    fn mini<T: MoveTimer>(
        &mut self,
        alpha: Score,
        mut beta: Score,
        depth: u16,
        timer: &T,
    ) -> SearchInfo {
        if depth == 0 || timer.over() {
            return SearchInfo {
                score: self.grade_position(),
                depth,
                nodes: 1,
            };
        }

        let existing = self.transposition_table.get(&self.game.hash);
        let better_than_existing = existing.is_none_or(|e| depth > e.depth);

        let mut result = SearchResult::new(Score::MAX, depth);

        for m in order_moves(self.game.legal_moves(), &existing) {
            let node = search_move!(self, &m, maxi(alpha, beta, depth - 1, timer));
            result += &node;

            if node.score < result.info.score {
                result.info.score = node.score;
                result.best_move = Some(m);
                if node.score < beta {
                    beta = node.score;
                }
            }

            if node.score <= alpha {
                break;
            }
        }

        if better_than_existing {
            let entry = TranspositionTableEntry {
                best_move: result.best_move,
                depth,
            };
            self.transposition_table.insert(self.game.hash, entry);
        }

        result.info
    }

    /// Continues searching at the given depth until the search finishes or the timer is over
    pub fn minimax<T: MoveTimer>(&mut self, timer: &T, depth: u16) -> SearchResult {
        let mut alpha = Score::MIN;
        let mut beta = Score::MAX;

        macro_rules! search_loop {
            ($best_score:expr, $cmp:tt, $search:ident, $prune:expr) => {{
                let existing = self.transposition_table.get(&self.game.hash);
                let better_than_existing = existing.is_none_or(|e| depth > e.depth);

                let mut result = SearchResult::new($best_score, 0);

                for m in order_moves(self.game.legal_moves(), &existing) {
                    let node = search_move!(self, &m, $search(alpha, beta, depth, timer));
                    result += &node;

                    if node.score $cmp result.info.score {
                        result.info.score = node.score;
                        result.best_move = Some(m);
                        if node.score $cmp $prune {
                            $prune = node.score;
                        }
                    }

                    if timer.over() {
                        break;
                    }
                }

                if better_than_existing {
                    let entry = TranspositionTableEntry { best_move: result.best_move, depth };
                    self.transposition_table.insert(self.game.hash, entry);
                }

                result
            }};
        }

        match self.game.turn {
            PieceColor::White => search_loop!(Score::MIN, >, mini, alpha),
            PieceColor::Black => search_loop!(Score::MAX, <, maxi, beta),
        }
    }

    /// Same as `search` but you can use your own timer
    pub fn search_with_timer<T: MoveTimer>(&mut self, timer: &T, max_depth: u16) -> SearchResult {
        let mut depth = 0;
        let mut result = SearchResult::default();

        loop {
            let node = self.minimax(timer, depth);
            result += &node;

            if node.best_move.is_none() || timer.over() {
                break;
            }

            result.best_move = node.best_move;
            result.info.score = node.info.score;

            if depth == max_depth {
                break;
            }
            depth += 1;
        }

        result
    }

    /// Searches for the best move in the position until the depth is reached or the duration is up
    pub fn search(&mut self, duration: Duration, max_depth: u16) -> SearchResult {
        if duration == Duration::MAX {
            self.search_with_timer(&Infinite, max_depth)
        } else {
            self.search_with_timer(&platform_timer!(duration), max_depth)
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
        let _ = engine.search_with_timer(&timer, u16::MAX);
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
        let best_move = engine.search(duration, u16::MAX).best_move;
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
        let result = engine
            .minimax(&Infinite, 2)
            .best_move
            .expect("No moves found");
        println!("State: {:?}", engine.game.state);
        assert_eq!(result, looking_for);
    }

    #[test]
    fn minimax_engine_saves_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 1 3";
        let mut engine = Engine::from_fen(starting).unwrap();
        let black_queens_before = engine.game.black_queens.popcnt();
        let result = engine
            .minimax(&Infinite, 2)
            .best_move
            .expect("No moves found");
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
            let result = engine.minimax(&Infinite, 0).best_move.unwrap();
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
            let result = engine.minimax(&Infinite, 1).best_move.unwrap();
            assert_eq!(result, looking_for);
            engine.game.unplay(&m);
        }
    }

    #[test]
    #[ignore]
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
        let _ = engine.minimax(&Infinite, 2).best_move;
        assert_eq!(before, engine.game);
    }

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

    #[test]
    fn engine_should_not_mutate_position() {
        let fen = "r1k2b1r/1p4p1/p1p4P/4B3/2p5/3P3P/NP2P1B1/2K2R2 w - - 0 29";
        let mut engine = Engine::from_fen(fen).unwrap();
        let before = engine.game.clone();
        let _ = engine.minimax(&Infinite, 3).best_move;
        let after = engine.game;
        assert_eq!(after, before);
    }

    #[test]
    fn should_have_moves_fen() {
        let fen = "rnbqkbnr/pp1ppppp/2p5/8/4PP2/8/PPPP2PP/RNBQKBNR b KQkq f3 0 2";
        let mut engine = Engine::from_fen(fen).unwrap();
        let moves = engine.game.legal_moves();
        let engine_move = engine.minimax(&Infinite, 2).best_move;
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
            let engine_move = engine.minimax(&Infinite, 2).best_move;
            assert_eq!(engine.game.state, State::InProgress);
            assert!(!moves.is_empty());
            assert!(engine_move.is_some())
        }
    }
}
