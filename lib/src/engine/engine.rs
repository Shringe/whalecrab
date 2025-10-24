use crate::{
    bitboard::BitBoard,
    board::State,
    engine::score::Score,
    game::Game,
    movegen::{
        moves::{Move, MoveType},
        pieces::piece::Color,
    },
    square::Square,
};

/// Plays a move, gets the score from the given method, and then unplays the move and returns that
/// score. Also does expensive validity checks in debug builds.
macro_rules! search_move {
    ($self:expr, $move:expr, $method:ident($($args:expr),*)) => {{
        #[cfg(debug_assertions)]
        let before = $self.position.clone();

        $move.play($self);
        $self.nodes_seached += 1;
        let score = $self.$method($($args),*);
        $move.unplay($self);

        #[cfg(debug_assertions)]
        assert_eq!(
            $self.position, before,
            "State changed after playing and unplaying {}",
            $move
        );

        score
    }};
}

/// Orders the moves for better minimax pruning
/// TODO: Figure out why the reduction in nodes searched is minimal. The outcome of the game is
/// also being changed sometimes
fn order_moves(mut moves: Vec<Move>) -> Vec<Move> {
    // return moves;

    moves.sort_unstable_by_key(|m| match m.variant {
        MoveType::Promotion(_) => 0,
        _ if m.capture.is_some() => 1,
        MoveType::Castle(_) => 2,
        _ => 3,
    });

    moves
}

impl Game {
    /// Score material based on its value and position on the board
    fn score_material(&self) -> Score {
        let mut score = Score::default();

        for sq in self.occupied {
            let sqbb = BitBoard::from_square(sq);
            let (piece, color) = self.determine_piece(&sqbb).unwrap();

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

    /// Scores king safety. Primarily based on whether the king has friendly pawns next to him.
    fn score_king_safety(&self) -> Score {
        let mut score = Score::default();

        let calculate_pawn_area = |king: &Square| {
            let file = king.get_file();
            let mut pawn_area = file.to_bitboard();
            if file.to_index() > 0 {
                pawn_area |= file.left().to_bitboard();
            }
            if file.to_index() < 7 {
                pawn_area |= file.right().to_bitboard();
            }
            pawn_area
        };

        let white_king = self.position.white_kings.to_square();
        let white_pawn_area = calculate_pawn_area(&white_king);
        score += Score::new(
            ((white_pawn_area & self.position.white_pawns).popcnt() * 15)
                .try_into()
                .unwrap(),
        );

        let black_king = self.position.black_kings.to_square();
        let black_pawn_area = calculate_pawn_area(&black_king);
        score -= Score::new(
            ((black_pawn_area & self.position.black_pawns).popcnt() * 15)
                .try_into()
                .unwrap(),
        );

        score
    }

    /// Scores the position castling rights
    fn score_castling_rights(&self) -> Score {
        let mut score = Score::default();
        let value = 2;

        if self.position.castling_rights.white_queenside {
            score += Score::new(value);
        }

        if self.position.castling_rights.white_kingside {
            score += Score::new(value);
        }

        if self.position.castling_rights.black_queenside {
            score -= Score::new(value);
        }

        if self.position.castling_rights.black_kingside {
            score -= Score::new(value);
        }

        score
    }

    /// Scores both attackers and defenders
    fn score_attackers(&self) -> Score {
        let mut score = Score::default();

        score += Score::new(
            ((self.white_attacks & self.occupied).popcnt() * 10)
                .try_into()
                .unwrap(),
        );
        score -= Score::new(
            ((self.black_attacks & self.occupied).popcnt() * 10)
                .try_into()
                .unwrap(),
        );

        score
    }

    /// Grades the postion. For example, -1.0 means black is wining by a pawn's worth of value
    pub fn grade_position(&mut self) -> Score {
        if let Some(pre) = self.transposition_table.get(&self.position.hash) {
            return *pre;
        }

        macro_rules! end {
            ($score: expr) => {{
                self.transposition_table.insert(self.position.hash, $score);
                return $score;
            }};
        }

        // State
        match self.position.state {
            State::InProgress => {}
            State::Checkmate => {
                end!(match self.position.turn {
                    Color::White => Score::MIN,
                    Color::Black => Score::MAX,
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
        for m in order_moves(self.generate_all_legal_moves()) {
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
        for m in order_moves(self.generate_all_legal_moves()) {
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

    pub fn get_engine_move_minimax(&mut self, depth: u16) -> Option<Move> {
        let moves = order_moves(self.generate_all_legal_moves());
        let mut best_move = None;

        let alpha = Score::MIN;
        let beta = Score::MAX;

        match self.position.turn {
            Color::White => {
                let mut best_score = Score::MIN;
                for m in moves {
                    let score = search_move!(self, m, mini(alpha, beta, depth));
                    if score > best_score {
                        best_score = score;
                        best_move = Some(m);
                    }
                }

                best_move
            }

            Color::Black => {
                let mut best_score = Score::MAX;
                for m in moves {
                    let score = search_move!(self, m, maxi(alpha, beta, depth));
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
    use std::time::{Duration, Instant};

    use super::*;
    use crate::{
        board::Board,
        movegen::{moves::MoveType, pieces::piece::PieceType},
        square::Square,
    };

    /// Used for determining cache hit/miss
    fn time_grading(game: &mut Game) -> (Score, Duration) {
        let start_time = Instant::now();
        let result = game.grade_position();
        let duration = start_time.elapsed();
        (result, duration)
    }

    #[test]
    fn starting_evaluation_is_balanced() {
        let mut game = Game::default();
        assert_eq!(game.grade_position(), Score::default());
    }

    #[test]
    fn minimax_engine_takes_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3";
        let mut game = Game::from_position(Board::from_fen(starting).unwrap());
        let looking_for = Move::new(Square::C1, Square::G5, &game.position);
        let result = game.get_engine_move_minimax(2).expect("No moves found");
        println!("State: {:?}", game.position.state);
        assert_eq!(result, looking_for);
    }

    #[test]
    fn minimax_engine_saves_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 1 3";
        let mut game = Game::from_position(Board::from_fen(starting).unwrap());
        let black_queens_before = game.position.black_queens.popcnt();
        let result = game.get_engine_move_minimax(2).expect("No moves found");
        game.play(&result);
        assert_eq!(black_queens_before, game.position.black_queens.popcnt());
    }

    #[test]
    fn black_always_takes_king() {
        let fen = "k6r/pp4r1/8/pp6/Qp6/pp6/7K/8 w - - 0 1";
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());
        let white_moves = game.generate_all_legal_moves();
        for m in white_moves {
            m.play(&mut game);
            let result = game.get_engine_move_minimax(0).unwrap();
            assert_eq!(result.variant, MoveType::Normal);
            assert_eq!(result.capture, Some(PieceType::King));
            m.unplay(&mut game);
        }
    }

    #[test]
    fn white_always_checkmates() {
        let fen = "7k/8/8/8/8/8/5R2/K5R1 b - - 0 1";
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());
        let black_moves = game.generate_all_legal_moves();
        for m in black_moves {
            m.play(&mut game);
            let looking_for = Move::new(Square::F2, Square::H2, &game.position);
            let result = game.get_engine_move_minimax(1).unwrap();
            assert_eq!(result, looking_for);
            m.unplay(&mut game);
        }
    }

    #[test]
    fn transportation_table_cache_hits() {
        let mut game = Game::default();

        let (initial_result, initial_duration) = time_grading(&mut game);
        let min_speedup_factor = 1.2;

        for i in 1..100 {
            let (result, duration) = time_grading(&mut game);
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
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());
        let before = game.clone();
        let _ = game.generate_all_legal_moves();
        let _ = game.get_engine_move_minimax(2);
        assert_eq!(before, game);
    }

    #[test]
    fn sort_moves_keeps_all_moves() {
        let mut game = Game::default();
        let moves = game.generate_all_legal_moves();
        let sorted = order_moves(moves.clone());
        for sortedm in &sorted {
            assert!(moves.contains(sortedm));
        }
        assert_eq!(sorted.len(), moves.len());
    }

    #[test]
    fn engine_should_not_mutate_position() {
        let fen = "r1k2b1r/1p4p1/p1p4P/4B3/2p5/3P3P/NP2P1B1/2K2R2 w - - 0 29";
        let mut game = Game::from_position(Board::from_fen(fen).unwrap());
        let before = game.position.clone();
        let _ = game.get_engine_move_minimax(3);
        let after = game.position;
        assert_eq!(before, after);
    }
}
