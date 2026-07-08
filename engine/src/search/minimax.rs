use crate::engine::Engine;
use crate::{move_result::SearchResult, timers::MoveTimer};

/// Plays a move, gets the score from the given method, and then unplays the move and returns that
/// score. Also does expensive validity checks in debug builds.
#[macro_export]
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

impl Engine {
    /// Continues searching at the given depth until the search finishes or the timer is over.
    ///
    /// Note: this now uses Negamax under the hood as of 1.6.0.
    /// This function will probably be deprecated in the future.
    pub fn minimax<T: MoveTimer>(&mut self, timer: &T, depth: u8) -> SearchResult {
        self.negamax(timer, depth)
    }
}

#[cfg(test)]
mod tests {
    use whalecrab_lib::{
        movegen::{moves::Move, pieces::piece::PieceColor},
        square::Square,
    };

    use crate::{move_result::SearchInfo, score::Score, timers::infinite::Infinite};

    use super::*;

    impl Engine {
        fn maxi_without_pruning<T: MoveTimer>(&mut self, depth: u8, timer: &T) -> SearchInfo {
            if depth == 0 || timer.over() {
                return SearchInfo {
                    score: self.grade_position(),
                    depth,
                    nodes: 1,
                };
            }

            let mut result = SearchResult::new(Score::MIN, depth);

            for m in self.game.legal_moves() {
                let node = search_move!(self, &m, mini_without_pruning(depth - 1, timer));
                result += &node;

                if node.score > result.info.score {
                    result.info.score = node.score;
                    result.best_move = Some(m);
                }
            }

            result.info
        }

        fn mini_without_pruning<T: MoveTimer>(&mut self, depth: u8, timer: &T) -> SearchInfo {
            if depth == 0 || timer.over() {
                return SearchInfo {
                    score: self.grade_position(),
                    depth,
                    nodes: 1,
                };
            }

            let mut result = SearchResult::new(Score::MAX, depth);

            for m in self.game.legal_moves() {
                let node = search_move!(self, &m, maxi_without_pruning(depth - 1, timer));
                result += &node;

                if node.score < result.info.score {
                    result.info.score = node.score;
                    result.best_move = Some(m);
                }
            }

            result.info
        }

        pub fn minimax_without_pruning<T: MoveTimer>(
            &mut self,
            timer: &T,
            depth: u8,
        ) -> SearchResult {
            macro_rules! search_loop {
            ($best_score:expr, $cmp:tt, $search:ident) => {{
                let mut result = SearchResult::new($best_score, 0);

                for m in self.game.legal_moves() {
                    let node = search_move!(self, &m, $search(depth, timer));
                    if timer.over() {
                        break;
                    }

                    result += &node;

                    if node.score $cmp result.info.score {
                        result.info.score = node.score;
                        result.best_move = Some(m);
                    }
                }

                result
            }};
        }

            match self.game.turn {
                PieceColor::White => search_loop!(Score::MIN, >, mini_without_pruning),
                PieceColor::Black => search_loop!(Score::MAX, <, maxi_without_pruning),
            }
        }
    }

    #[track_caller]
    fn assert_minimax_pruning_is_lossless(engine: &mut Engine, depth: u8) {
        let actual = engine.minimax(&Infinite, depth);
        let expected = engine.minimax_without_pruning(&Infinite, depth);
        assert_eq!(
            actual, expected,
            "Minimax pruning is not lossless at depth {}",
            depth
        );
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

    #[ignore]
    #[test]
    fn canary_minimax_pruning_should_be_lossless() {
        let fen = "k7/pp6/4n3/8/3K1Q2/8/8/R7 w - - 1 2";
        let mut engine = Engine::from_fen(fen).unwrap();
        for depth in 0..=5 {
            assert_minimax_pruning_is_lossless(&mut engine, depth);
        }
    }

    #[ignore]
    #[test]
    fn canary_minimax_pruning_should_be_lossless_depth_3_to_4() {
        let mut engine = Engine::default();
        assert_minimax_pruning_is_lossless(&mut engine, 3);
        assert_minimax_pruning_is_lossless(&mut engine, 4);
    }
}
