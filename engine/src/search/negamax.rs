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
        self.nega(timer, depth, Score::MIN, Score::MAX)
    }
}
