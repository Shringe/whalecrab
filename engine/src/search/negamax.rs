use crate::{
    engine::Engine,
    move_result::{SearchInfo, SearchResult},
    score::Score,
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
        beta: Score,
    ) -> SearchResult {
        if depth == 0 || timer.over() {
            return SearchResult {
                info: SearchInfo {
                    score: self.grade_position_relative(),
                    depth,
                    nodes: 1,
                },
                best_move: None,
            };
        }

        let existing = self.transposition_table.get(self.game.hash);
        let better_than_existing = if let Some(entry) = &existing {
            if depth == entry.depth {
                return SearchResult {
                    info: SearchInfo {
                        score: entry.score,
                        depth,
                        nodes: 1,
                    },
                    best_move: entry.best_move,
                };
            } else if depth > entry.depth {
                if alpha > entry.score {
                    alpha = entry.score;
                }
                false
            } else {
                true
            }
        } else {
            true
        };

        let mut node_type = NodeType::Exact;
        let mut result = SearchResult::new(Score::MIN, depth);

        for m in order_moves(self.game.legal_moves(), &existing) {
            let mut node = search_move!(self, &m, nega(timer, depth - 1, -beta, -alpha));
            node.info.score = -node.info.score;
            result += &node.info;

            if node.info.score > result.info.score {
                result.info.score = node.info.score;
                result.best_move = Some(m);
                if node.info.score > alpha {
                    alpha = node.info.score;
                }
            }

            if node.info.score >= beta {
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

        result
    }

    /// Continues searching at the given depth until the search finishes or the timer is over
    pub fn negamax<T: MoveTimer>(&mut self, timer: &T, depth: u8) -> SearchResult {
        self.nega(timer, depth + 1, Score::MIN, Score::MAX)
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
        assert_eq!(nresult.best_move, mresult.best_move);
        assert_eq!(nresult.info.score, mresult.info.score);
        if !cache {
            engine.clear_persistant_cache();
            assert_eq!(nresult.info.nodes, mresult.info.nodes);
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
                nresult.best_move, mresult.best_move,
                "Minimax: {:#?}\nNegamax: {:#?}\nGame: {:?}\n",
                mresult, nresult, mini.game
            );

            let Some(m) = mresult.best_move else {
                break;
            };

            mini.game.play(&m);
            nega.game.play(&m);
        }
    }
}
