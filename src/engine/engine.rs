use std::{cmp::Ordering, fmt};

use crate::{
    board::{Board, Color},
    movegen::moves::Move,
};

#[derive(Clone)]
pub struct ScoredMove(pub Move, f32);

impl fmt::Display for ScoredMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {}, {:?}, {}",
            self.0.from, self.0.to, self.0.variant, self.1
        )
    }
}

impl fmt::Debug for ScoredMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Ord for ScoredMove {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.partial_cmp(&other.1).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for ScoredMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScoredMove {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl Eq for ScoredMove {}

impl ScoredMove {
    pub fn from_moves(board: &Board, moves: Vec<Move>) -> Vec<Self> {
        let mut scored_moves = Vec::new();
        for m in moves {
            let new = m.make(board);
            let score = new.grade_position();
            let scored_move = ScoredMove(m, score);
            scored_moves.push(scored_move);
        }

        scored_moves
    }
}

impl Board {
    /// Grades the postion. For example, -1.0 means black is wining by a pawn's worth of value
    pub fn grade_position(&self) -> f32 {
        let mut score = 0.0;

        // Piece value
        for sq in self.occupied_bitboard() {
            let piece = self.determine_piece(sq).unwrap();
            let color = self.determine_color(sq).unwrap();

            match color {
                Color::White => {
                    score += piece.material_value();
                    score += piece.square_value(&sq, &color);
                }

                Color::Black => {
                    score -= piece.material_value();
                    score -= piece.square_value(&sq, &color);
                }
            }
        }

        score
    }

    /// Finds and returns the suggested move for the current players turn
    pub fn find_best_move(&self) -> Option<ScoredMove> {
        let moves = self.generate_all_legal_moves();
        let mut scored_moves = ScoredMove::from_moves(self, moves);
        scored_moves.sort();
        match self.turn {
            Color::White => scored_moves.into_iter().next_back(),
            Color::Black => scored_moves.into_iter().next(),
        }
    }

    /// Recusively searches through the specified depth (in half moves) to find the best move
    pub fn get_engine_move(&self, depth: u16, initial: Option<ScoredMove>) -> Option<ScoredMove> {
        if depth == 0 {
            let best = self.find_best_move();
            return best;
        }

        let moves = self.generate_all_legal_moves();
        let scored_moves = ScoredMove::from_moves(self, moves);
        let mut best_initial = None;
        let mut best_final = None;
        for m in scored_moves {
            let board = m.0.make(self);
            let initial = if initial.is_some() {
                initial.clone()
            } else {
                Some(m)
            };

            if let Some(m) = board.get_engine_move(depth - 1, initial.clone()) {
                if let Some(b) = &best_final {
                    if &m > b {
                        best_initial = initial;
                        best_final = Some(m);
                    }
                } else {
                    best_initial = initial;
                    best_final = Some(m)
                }
            }
        }

        best_initial
    }

    // https://www.chessprogramming.org/Minimax
    // int maxi( int depth ) {
    //     if ( depth == 0 ) return evaluate();
    //     int max = -oo;
    //     for ( all moves) {
    //         score = mini( depth - 1 );
    //         if( score > max )
    //             max = score;
    //     }
    //     return max;
    // }
    //
    // int mini( int depth ) {
    //     if ( depth == 0 ) return -evaluate();
    //     int min = +oo;
    //     for ( all moves) {
    //         score = maxi( depth - 1 );
    //         if( score < min )
    //             min = score;
    //     }
    //     return min;
    // }
    fn maxi(&self, depth: u16) -> f32 {
        if depth == 0 {
            return self.grade_position();
        }

        let mut max = f32::MIN;
        for m in self.generate_all_legal_moves() {
            let potential = m.make(self);
            let score = potential.mini(depth - 1);
            if score > max {
                max = score;
            }
        }

        max
    }

    fn mini(&self, depth: u16) -> f32 {
        if depth == 0 {
            return self.grade_position();
        }

        let mut min = f32::MAX;
        for m in self.generate_all_legal_moves() {
            let potential = m.make(self);
            let score = potential.maxi(depth - 1);
            if score < min {
                min = score;
            }
        }

        min
    }

    pub fn get_engine_move_minimax(&self, depth: u16) -> Option<Move> {
        let moves = self.generate_all_legal_moves();
        let mut best_move = None;

        match self.turn {
            Color::White => {
                let mut best_score = f32::MIN;
                for m in moves {
                    let potential = m.make(self);
                    let score = potential.mini(depth - 1);
                    if score > best_score {
                        best_score = score;
                        best_move = Some(m);
                    }
                }

                best_move
            }

            Color::Black => {
                let mut best_score = f32::MAX;
                for m in moves {
                    let potential = m.make(self);
                    let score = potential.maxi(depth - 1);
                    if score < best_score {
                        best_score = score;
                        best_move = Some(m);
                    }
                }

                best_move
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::Square;

    #[test]
    fn old_engine_takes_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3";
        let board = Board::from_fen(starting).unwrap();
        let looking_for = Move::new(Square::C1, Square::G5, &board);
        let result = board.find_best_move().expect("No moves found");
        assert_eq!(result.0, looking_for);
    }

    #[test]
    fn old_engine_saves_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 1 3";
        let board = Board::from_fen(starting).unwrap();
        let result = board.find_best_move().expect("No moves found");
        let new = result.0.make(&board);
        assert_eq!(
            board.black_queen_bitboard.popcnt(),
            new.black_queen_bitboard.popcnt()
        );
    }

    #[test]
    fn minimax_engine_takes_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3";
        let board = Board::from_fen(starting).unwrap();
        let looking_for = Move::new(Square::C1, Square::G5, &board);
        let result = board.get_engine_move_minimax(2).expect("No moves found");
        let moves = board.generate_all_legal_moves();
        let mut scored = ScoredMove::from_moves(&board, moves);
        scored.sort();
        println!("Available moves: {:#?}", scored);
        assert_eq!(result, looking_for);
    }

    #[test]
    fn minimax_engine_saves_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 1 3";
        let board = Board::from_fen(starting).unwrap();
        let result = board.get_engine_move_minimax(2).expect("No moves found");
        let new = result.make(&board);
        assert_eq!(
            board.black_queen_bitboard.popcnt(),
            new.black_queen_bitboard.popcnt()
        );
    }
}
