use std::fmt;

use whalecrab_lib::movegen::moves::Move;

use crate::score::Score;

/// The reason that a search ended
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum Terminal {
    /// The search fully completed its depth target
    #[default]
    Depth,
    /// The search ran out of time
    Timer,
}

/// Provides relevant information about the completed search
#[derive(Debug, Default)]
pub struct SearchResult {
    /// The best move found in the search
    pub best: Option<Move>,
    /// The reason the search ended
    pub terminal: Terminal,
    /// The best score from a search
    pub score: Score,
    /// The maximum depth reached in a search
    pub depth: u8,
    /// The number of nodes a searched evaluated
    pub nodes: u64,
}

impl PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        self.best == other.best
            && self.terminal == other.terminal
            && self.score == other.score
            && self.depth == other.depth
    }
}

impl fmt::Display for SearchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "score: {}\ndepth: {}\nnodes: {}",
            self.score, self.depth, self.nodes
        )
    }
}

impl SearchResult {
    pub const fn new(score: Score, depth: u8) -> Self {
        Self {
            best: None,
            terminal: Terminal::Depth,
            score,
            depth,
            nodes: 1,
        }
    }

    /// Creates
    pub fn leaf(score: Score, depth: u8, terminal: Terminal) -> Self {
        Self {
            best: None,
            terminal,
            score,
            depth,
            nodes: 1,
        }
    }
}
