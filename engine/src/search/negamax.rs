use crate::{
    engine::Engine,
    move_result::{SearchResult, Terminal},
    score::{Score, ScoreColor},
    search::move_ordering::order_moves,
    search_move,
    timers::MoveTimer,
    transposition_table::{NodeType, TranspositionTableEntry},
};

impl Engine {
    fn nega<T: MoveTimer>(
        &mut self,
        timer: &T,
        depth: u8,
        mut alpha: Score,
        mut beta: Score,
        color: ScoreColor,
    ) -> SearchResult {
        if depth == 0 {
            return SearchResult::leaf(self.grade_position() * color, depth, Terminal::Depth);
        } else if timer.over() {
            return SearchResult::leaf(self.grade_position() * color, depth, Terminal::Timer);
        }

        let existing = self.transposition_table.get(self.game.hash);
        if let Some(entry) = &existing
            && entry.depth >= depth
        {
            match entry.node_type {
                NodeType::Exact => {
                    return SearchResult {
                        best: entry.best,
                        terminal: Terminal::Depth,
                        score: entry.score,
                        depth,
                        nodes: 1,
                    };
                }
                NodeType::LowerBound => {
                    if entry.score >= beta {
                        return SearchResult {
                            best: entry.best,
                            terminal: Terminal::Depth,
                            score: entry.score,
                            depth,
                            nodes: 1,
                        };
                    }
                    alpha = alpha.max(entry.score);
                }
                NodeType::UpperBound => {
                    if entry.score < alpha {
                        return SearchResult {
                            best: entry.best,
                            terminal: Terminal::Depth,
                            score: entry.score,
                            depth,
                            nodes: 1,
                        };
                    }
                    beta = beta.min(entry.score);
                }
            }
            if alpha >= beta {
                return SearchResult {
                    best: entry.best,
                    terminal: Terminal::Depth,
                    score: entry.score,
                    depth,
                    nodes: 1,
                };
            }
        }

        let original_alpha = alpha;
        let mut node_type = NodeType::Exact;
        let mut result = SearchResult::new(Score::MIN, depth);

        'search: {
            macro_rules! evaluate {
                ($m:expr) => {{
                    let mut child =
                        search_move!(self, &$m, nega(timer, depth - 1, -beta, -alpha, -color));
                    result.nodes += child.nodes;

                    if child.terminal == Terminal::Timer {
                        result.terminal = Terminal::Timer;
                        return result;
                    }

                    child.score = -child.score;

                    if child.score > result.score {
                        result.score = child.score;
                        result.best = Some($m);
                        alpha = alpha.max(child.score);
                    }

                    if child.score >= beta {
                        node_type = NodeType::LowerBound;
                        break 'search;
                    }
                }};
            }

            if let Some(m) = existing.as_ref().and_then(|e| e.best) {
                evaluate!(m);
            }

            for m in order_moves(self.game.legal_moves(), &existing) {
                evaluate!(m);
            }
        }

        if node_type != NodeType::LowerBound && alpha == original_alpha {
            node_type = NodeType::UpperBound;
        }

        let better_than_existing = self
            .transposition_table
            .get(self.game.hash)
            .is_none_or(|e| e.depth < depth);

        if better_than_existing {
            let entry = TranspositionTableEntry {
                best: result.best,
                depth,
                score: result.score,
                node_type,
            };
            self.transposition_table.insert(self.game.hash, entry);
        }

        result
    }

    /// Continues searching at the given depth until the search finishes or the timer is over
    pub fn negamax<T: MoveTimer>(&mut self, timer: &T, depth: u8) -> SearchResult {
        self.nega(
            timer,
            depth + 1,
            Score::MIN,
            Score::MAX,
            ScoreColor::from_color(self.game.turn),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::timers::infinite::Infinite;

    use super::*;

    #[track_caller]
    fn assert_nem(engine: &mut Engine, depth: u8, cache: bool) -> SearchResult {
        let start = Instant::now();
        let mresult = engine.minimax(&Infinite, depth);
        let mtime = start.elapsed();

        if !cache {
            engine.clear_persistant_cache();
        }

        let start = Instant::now();
        let nresult = engine.negamax(&Infinite, depth);
        let ntime = start.elapsed();

        println!("Minimax took {:?}; Negamax took {:?}", mtime, ntime);
        assert_eq!(nresult.best, mresult.best);
        assert_eq!(nresult.score, mresult.score);
        if !cache {
            engine.clear_persistant_cache();
            assert_eq!(nresult.nodes, mresult.nodes);
        }
        mresult
    }

    #[test]
    fn negamax_result_equals_minimax_result_default_position() {
        assert_nem(&mut Engine::default(), 1, false);
    }

    #[ignore]
    #[test]
    fn canary_negamax_result_equals_minimax_result_default_position_many_depths() {
        for depth in 1..5 {
            println!("Depth: {}", depth);
            assert_nem(&mut Engine::default(), depth, false);
        }
    }

    #[ignore]
    #[test]
    fn canary_negamax_result_equals_minimax_result_full_game() {
        let mut mini = Engine::default();
        let mut nega = Engine::default();
        let depth = 2;

        loop {
            assert_eq!(mini.game, nega.game);

            let mresult = mini.minimax(&Infinite, depth);
            let nresult = nega.negamax(&Infinite, depth);
            assert_eq!(
                nresult.best, mresult.best,
                "Minimax: {:#?}\nNegamax: {:#?}\nGame: {:?}\n",
                mresult, nresult, mini.game
            );

            let Some(m) = mresult.best else {
                break;
            };

            mini.game.play(&m);
            nega.game.play(&m);
        }
    }
}
