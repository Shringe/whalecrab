use std::{fmt, ops::AddAssign};

use whalecrab_lib::movegen::moves::Move;

use crate::score::Score;

/// Provides relevant information about the completed search
#[derive(Debug)]
pub struct SearchInfo {
    /// The best score from a search
    pub score: Score,
    /// The maximum depth reached in a search
    pub depth: u16,
    /// The number of nodes a searched evaluated
    pub nodes: u64,
}

impl PartialEq for SearchInfo {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.depth == other.depth
    }
}

impl SearchInfo {
    pub const fn new(score: Score, depth: u16) -> Self {
        Self {
            score,
            depth,
            nodes: 1,
        }
    }
}

impl Default for SearchInfo {
    fn default() -> Self {
        Self {
            score: Score::default(),
            depth: 0,
            nodes: 1,
        }
    }
}

impl AddAssign<&SearchInfo> for SearchInfo {
    fn add_assign(&mut self, rhs: &SearchInfo) {
        self.nodes += rhs.nodes;
        if rhs.depth > self.depth {
            self.depth = rhs.depth;
        }
    }
}

impl fmt::Display for SearchInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "score: {}\ndepth: {}\nnodes: {}",
            self.score, self.depth, self.nodes
        )
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub info: SearchInfo,
}

impl SearchResult {
    pub const fn new(score: Score, depth: u16) -> SearchResult {
        SearchResult {
            best_move: None,
            info: SearchInfo::new(score, depth),
        }
    }
}

impl AddAssign<&SearchInfo> for SearchResult {
    fn add_assign(&mut self, rhs: &SearchInfo) {
        self.info += rhs;
    }
}

impl AddAssign<&SearchResult> for SearchResult {
    fn add_assign(&mut self, rhs: &SearchResult) {
        self.info += &rhs.info;
    }
}

impl fmt::Display for SearchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Best: {:?}\n{}", self.best_move, self.info)
    }
}
