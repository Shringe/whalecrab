use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use crate::{
    piece_eval::{material_value, square_value},
    score::Score,
};
use dashmap::DashMap;
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

#[derive(Default, Clone, Debug)]
pub struct Engine {
    /// Use self.with_new_game(game) instead of self.game = game if you want to replace this value
    pub game: Game,
    transposition_table: Arc<DashMap<u64, Score>>,
    pub nodes_searched: u64,
}

impl Engine {
    pub fn from_game(game: Game) -> Engine {
        Engine {
            game,
            transposition_table: Arc::new(DashMap::new()),
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

    fn maxi(&mut self, mut alpha: Score, beta: Score, depth: u16) -> Score {
        if depth == 0 {
            return self.grade_position();
        }

        let mut max = Score::MIN;
        for m in order_moves(self.game.legal_moves()) {
            let score = search_move!(self, m, mini(alpha, beta, depth - 1));
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

    fn mini(&mut self, alpha: Score, mut beta: Score, depth: u16) -> Score {
        if depth == 0 {
            return self.grade_position();
        }

        let mut min = Score::MAX;
        for m in order_moves(self.game.legal_moves()) {
            let score = search_move!(self, m, maxi(alpha, beta, depth - 1));
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
        self.minimax_with_duration(depth, &Instant::now(), &Duration::MAX)
            .0
    }

    /// Continues searching until either the depth or duration is reached
    pub fn minimax_with_duration_single_threaded(
        &mut self,
        depth: u16,
        since: &Instant,
        duration: &Duration,
    ) -> (Option<Move>, bool) {
        let moves = order_moves(self.game.legal_moves());
        let mut best_move = None;

        let alpha = Score::MIN;
        let beta = Score::MAX;

        macro_rules! search_loop {
            ($best_score:expr, $cmp:tt, $search:ident) => {{
                let mut best_score = $best_score;
                for m in moves {
                    let score = search_move!(self, m, $search(alpha, beta, depth));
                    if score $cmp best_score {
                        best_score = score;
                        best_move = Some(m);
                    }

                    let finished = Instant::now();
                    let elapsed = finished.duration_since(*since);
                    if elapsed > *duration {
                        return (best_move, true);
                    }
                }
                (best_move, false)
            }};
        }

        match self.game.turn {
            PieceColor::White => search_loop!(Score::MIN, >, mini),
            PieceColor::Black => search_loop!(Score::MAX, <, maxi),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn maxi_threaded(
        &mut self,
        mut alpha: Score,
        beta: Score,
        depth: u16,
        cancel_search: Arc<AtomicBool>,
        best: Arc<Mutex<(Score, Option<Move>)>>,
        nodes: Arc<AtomicU64>,
        num_active_threads: Arc<AtomicU8>,
        num_threads: u8,
        root_move: Move,
        since: Instant,
        duration: Duration,
        spawn_threshold: Arc<AtomicU8>,
    ) -> Score {
        if depth == 0 {
            return self.grade_position();
        } else if cancel_search.load(Ordering::Relaxed) {
            return Score::MAX;
        } else if since.elapsed() > duration {
            cancel_search.store(true, Ordering::Relaxed);
            return Score::MAX;
        }

        let mut max = Score::MIN;
        let mut max2 = max;
        let mut m2 = None;

        for m in order_moves(self.game.legal_moves()) {
            let score = search_move!(
                self,
                m,
                mini_threaded(
                    alpha,
                    beta,
                    depth - 1,
                    cancel_search.clone(),
                    best.clone(),
                    nodes.clone(),
                    num_active_threads.clone(),
                    num_threads,
                    root_move,
                    since,
                    duration,
                    spawn_threshold.clone()
                )
            );

            if score > max {
                max = score;
                if score > alpha {
                    alpha = score;
                }
            } else if score > max2 {
                max2 = score;
                m2 = Some(m);
            }

            if score >= beta {
                break;
            }
        }

        // Spawn a new thread for the second best move if its within spawn_threshold and there's a
        // thread available
        if let Some(m2) = m2 {
            let threshold = spawn_threshold.load(Ordering::Relaxed);
            if max.saturating_sub(max2) < Score::new(threshold as i32) {
                if num_active_threads
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                        if n < num_threads { Some(n + 1) } else { None }
                    })
                    .is_ok()
                {
                    let mut engine = self.clone();
                    engine.nodes_searched = 0;
                    let best = best.clone();
                    let cancel_search = cancel_search.clone();
                    let num_active_threads = num_active_threads.clone();
                    let nodes = nodes.clone();

                    let _ = thread::spawn(move || {
                        let score = search_move!(
                            engine,
                            m2,
                            mini_threaded(
                                alpha,
                                beta,
                                depth - 1,
                                cancel_search,
                                best.clone(),
                                nodes.clone(),
                                num_active_threads.clone(),
                                num_threads,
                                root_move,
                                since,
                                duration,
                                spawn_threshold
                            )
                        );

                        if score > max {
                            let mut best = best.lock().unwrap();
                            if score > best.0 {
                                *best = (score, Some(root_move));
                            }
                        }

                        let _ = nodes.fetch_add(engine.nodes_searched, Ordering::Relaxed);
                        let _ = num_active_threads.fetch_sub(1, Ordering::Relaxed);
                    });
                } else if threshold > u8::MIN + 4 {
                    spawn_threshold.store(threshold - 5, Ordering::Relaxed);
                }
            } else if threshold < u8::MAX {
                spawn_threshold.store(threshold + 1, Ordering::Relaxed);
            }
        }

        max
    }

    #[allow(clippy::too_many_arguments)]
    fn mini_threaded(
        &mut self,
        alpha: Score,
        mut beta: Score,
        depth: u16,
        cancel_search: Arc<AtomicBool>,
        best: Arc<Mutex<(Score, Option<Move>)>>,
        nodes: Arc<AtomicU64>,
        num_active_threads: Arc<AtomicU8>,
        num_threads: u8,
        root_move: Move,
        since: Instant,
        duration: Duration,
        spawn_threshold: Arc<AtomicU8>,
    ) -> Score {
        if depth == 0 {
            return self.grade_position();
        } else if cancel_search.load(Ordering::Relaxed) {
            return Score::MAX;
        } else if since.elapsed() > duration {
            cancel_search.store(true, Ordering::Relaxed);
            return Score::MAX;
        }

        let mut min = Score::MAX;
        let mut min2 = min;
        let mut m2 = None;

        for m in order_moves(self.game.legal_moves()) {
            let score = search_move!(
                self,
                m,
                maxi_threaded(
                    alpha,
                    beta,
                    depth - 1,
                    cancel_search.clone(),
                    best.clone(),
                    nodes.clone(),
                    num_active_threads.clone(),
                    num_threads,
                    root_move,
                    since,
                    duration,
                    spawn_threshold.clone()
                )
            );

            if score < min {
                min = score;
                if score < beta {
                    beta = score;
                }
            } else if score < min2 {
                min2 = score;
                m2 = Some(m);
            }

            if score <= alpha {
                break;
            }
        }

        // Spawn a new thread for the second best move if its within spawn_threshold and there's a
        // thread available
        if let Some(m2) = m2 {
            let threshold = spawn_threshold.load(Ordering::Relaxed);
            if min2.saturating_sub(min) < Score::new(threshold as i32) {
                if num_active_threads
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                        if n < num_threads { Some(n + 1) } else { None }
                    })
                    .is_ok()
                {
                    let mut engine = self.clone();
                    engine.nodes_searched = 0;
                    let best = best.clone();
                    let cancel_search = cancel_search.clone();
                    let num_active_threads = num_active_threads.clone();
                    let nodes = nodes.clone();

                    let _ = thread::spawn(move || {
                        let score = search_move!(
                            engine,
                            m2,
                            maxi_threaded(
                                alpha,
                                beta,
                                depth - 1,
                                cancel_search,
                                best.clone(),
                                nodes.clone(),
                                num_active_threads.clone(),
                                num_threads,
                                root_move,
                                since,
                                duration,
                                spawn_threshold
                            )
                        );

                        if score < min {
                            let mut best = best.lock().unwrap();
                            if score < best.0 {
                                *best = (score, Some(root_move));
                            }
                        }

                        let _ = nodes.fetch_add(engine.nodes_searched, Ordering::Relaxed);
                        let _ = num_active_threads.fetch_sub(1, Ordering::Relaxed);
                    });
                } else if threshold > u8::MIN + 4 {
                    spawn_threshold.store(threshold - 5, Ordering::Relaxed);
                }
            } else if threshold < u8::MAX {
                spawn_threshold.store(threshold + 1, Ordering::Relaxed);
            }
        }

        min
    }

    /// Continues searching until either the depth or duration is reached
    pub fn minimax_with_duration_threaded(
        &mut self,
        depth: u16,
        since: Instant,
        duration: Duration,
        num_threads: u8,
        wait_for_children: bool,
    ) -> (Option<Move>, bool) {
        let moves = order_moves(self.game.legal_moves());

        let num_active_threads = Arc::new(AtomicU8::new(0));
        let nodes = Arc::new(AtomicU64::new(0));
        let cancel_search = Arc::new(AtomicBool::new(false));

        let alpha = Score::MIN;
        let beta = Score::MAX;

        macro_rules! search_loop {
            ($best_score:expr, $cmp:tt, $search:ident) => {{
                let best = Arc::new(Mutex::new(($best_score, None)));

                macro_rules! end {
                    ($timed_out: expr) => {{
                        self.nodes_searched += nodes.load(Ordering::Relaxed);
                        return match best.lock() {
                            Ok(m) => (m.1, $timed_out),
                            Err(_) => (None, $timed_out),
                        };
                    }};
                }

                for m in moves {
                    if cancel_search.load(Ordering::Relaxed) {
                        end!(true);
                    } else if since.elapsed() > duration {
                        cancel_search.store(true, Ordering::Relaxed);
                        end!(true);
                    }

                    let score = search_move!(self, m, $search(alpha, beta, depth, cancel_search.clone(), best.clone(), nodes.clone(), num_active_threads.clone(), num_threads, m, since, duration, Arc::new(AtomicU8::new(50))));

                    {
                        let mut best = best.lock().unwrap();
                        if score $cmp best.0 {
                            *best = (score, Some(m));
                        }
                    }
                }

                if !wait_for_children {
                    cancel_search.store(true, Ordering::Relaxed);
                    let timed_out = since.elapsed() > duration;
                    end!(timed_out);
                }

                let mut timed_out = false;
                loop {
                    if !timed_out && since.elapsed() > duration {
                        // Let all threads deposit their best values and finish their node
                        cancel_search.store(true, Ordering::Relaxed);
                        timed_out = true;
                    }
                    if num_active_threads.load(Ordering::Relaxed) == 0 {
                        break;
                    }
                    thread::sleep(Duration::from_millis(1));
                }
                end!(timed_out);
            }};
        }

        match self.game.turn {
            PieceColor::White => search_loop!(Score::MIN, >, mini_threaded),
            PieceColor::Black => search_loop!(Score::MAX, <, maxi_threaded),
        }
    }

    /// Continues searching until either the depth or duration is reached
    pub fn minimax_with_duration(
        &mut self,
        depth: u16,
        since: &Instant,
        duration: &Duration,
    ) -> (Option<Move>, bool) {
        // self.minimax_with_duration_single_threaded(depth, since, duration)
        self.minimax_with_duration_threaded(depth, *since, *duration, 16, true)
    }

    /// The engine will continue searching deeper and deeper depths until the duration has passed,
    /// at which point it will return the best move found so far.
    pub fn iterative_deepening(&mut self, duration: &Duration) -> Option<Move> {
        let start = Instant::now();
        let mut depth = 0;
        let mut best_move_so_far = None;

        loop {
            let (best_move, timed_out) = self.minimax_with_duration(depth, &start, duration);

            #[cfg(debug_assertions)]
            {
                let ran_out_of_time = start.elapsed() > *duration;
                assert_eq!(timed_out, ran_out_of_time);
            }

            if timed_out {
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

    use super::*;
    use whalecrab_lib::{movegen::pieces::piece::PieceType, square::Square};

    /// Used for determining cache hit/miss
    fn time_grading(engine: &mut Engine) -> (Score, Duration) {
        let start_time = Instant::now();
        let result = engine.grade_position();
        let duration = start_time.elapsed();
        (result, duration)
    }

    #[test]
    fn iterative_deepening_should_not_take_too_long() {
        let mut engine = Engine::default();
        let now = Instant::now();
        let duration = Duration::from_millis(200);
        let _ = engine.iterative_deepening(&duration);
        let elapsed = now.elapsed();
        let max = duration * 2;
        assert!(
            elapsed < max,
            "iterative_deepening for {:?} should have completed within {:?}, but took {:?}",
            duration,
            max,
            elapsed
        );
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
