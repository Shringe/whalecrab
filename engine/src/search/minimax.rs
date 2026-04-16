use whalecrab_lib::movegen::pieces::piece::PieceColor;

use crate::engine::Engine;
use crate::score::Score;
use crate::search::move_ordering::order_moves;
use crate::transposition_table::{NodeType, TranspositionTableEntry};
use crate::{
    move_result::{SearchInfo, SearchResult},
    timers::MoveTimer,
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

impl Engine {
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
        let better_than_existing = if let Some(entry) = existing {
            if depth == entry.depth {
                return SearchInfo {
                    score: entry.score,
                    depth,
                    nodes: 1,
                };
            }

            if entry.node_type == NodeType::Cut {
                alpha = alpha.max(entry.score);
            }

            depth > entry.depth
        } else {
            true
        };

        let mut node_type = NodeType::Exact;
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
                node_type = NodeType::Cut;
                break;
            }
        }

        if better_than_existing {
            let entry = TranspositionTableEntry {
                best_move: result.best_move,
                depth,
                score: result.info.score,
                node_type,
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
        let better_than_existing = if let Some(entry) = existing {
            if depth == entry.depth {
                return SearchInfo {
                    score: entry.score,
                    depth,
                    nodes: 1,
                };
            }

            if entry.node_type == NodeType::All {
                beta = beta.min(entry.score);
            }

            depth > entry.depth
        } else {
            true
        };

        let mut node_type = NodeType::Exact;
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
                node_type = NodeType::All;
                break;
            }
        }

        if better_than_existing {
            let entry = TranspositionTableEntry {
                best_move: result.best_move,
                depth,
                score: result.info.score,
                node_type,
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
                    if timer.over() {
                        break;
                    }

                    result += &node;

                    if node.score $cmp result.info.score {
                        result.info.score = node.score;
                        result.best_move = Some(m);
                        if node.score $cmp $prune {
                            $prune = node.score;
                        }
                    }
                }

                if better_than_existing {
                    let entry = TranspositionTableEntry {
                        best_move: result.best_move,
                        depth,
                        score: result.info.score,
                        node_type: NodeType::Exact,
                    };
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
}

#[cfg(test)]
mod test {
    use whalecrab_lib::{movegen::moves::Move, square::Square};

    use crate::timers::infinite::Infinite;

    use super::*;

    impl Engine {
        fn maxi_without_pruning<T: MoveTimer>(&mut self, depth: u16, timer: &T) -> SearchInfo {
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

        fn mini_without_pruning<T: MoveTimer>(&mut self, depth: u16, timer: &T) -> SearchInfo {
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
            depth: u16,
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
        let mut engine = Engine::default();
        for depth in 0..5 {
            let actual = engine.minimax(&Infinite, depth);
            let expected = engine.minimax_without_pruning(&Infinite, depth);
            assert_eq!(
                actual, expected,
                "Minimax pruning is not lossless at depth {}",
                depth
            );
        }
    }

    #[ignore]
    #[test]
    fn canary_minimax_pruning_should_be_lossless_depth_4() {
        let mut engine = Engine::default();
        let depth = 4;
        let actual = engine.minimax(&Infinite, depth);
        let expected = engine.minimax_without_pruning(&Infinite, depth);
        assert_eq!(
            actual, expected,
            "Minimax pruning is not lossless at depth {}",
            depth
        );
    }
}
