use crate::{
    engine::Engine,
    piece_eval::{material_value, square_value},
    score::Score,
};
use whalecrab_lib::{
    file::File,
    movegen::pieces::piece::{PieceColor, PieceType},
    position::game::State,
    square::Square,
};

impl Engine {
    fn score_white_material(&self) -> Score {
        let mut score = Score::default();

        score += material_value(PieceType::Pawn) * self.game.white_pawns.popcnt() as i32;
        score += material_value(PieceType::Knight) * self.game.white_knights.popcnt() as i32;
        score += material_value(PieceType::Bishop) * self.game.white_bishops.popcnt() as i32;
        score += material_value(PieceType::Rook) * self.game.white_rooks.popcnt() as i32;
        score += material_value(PieceType::Queen) * self.game.white_queens.popcnt() as i32;

        score
    }

    fn score_black_material(&self) -> Score {
        let mut score = Score::default();

        score += material_value(PieceType::Pawn) * self.game.black_pawns.popcnt() as i32;
        score += material_value(PieceType::Knight) * self.game.black_knights.popcnt() as i32;
        score += material_value(PieceType::Bishop) * self.game.black_bishops.popcnt() as i32;
        score += material_value(PieceType::Rook) * self.game.black_rooks.popcnt() as i32;
        score += material_value(PieceType::Queen) * self.game.black_queens.popcnt() as i32;

        score
    }

    fn midgame_to_lategame_ratio(&self, total_material: Score) -> f64 {
        let max_material = material_value(PieceType::Queen) * 1
            + material_value(PieceType::Rook) * 2
            + material_value(PieceType::Bishop) * 2
            + material_value(PieceType::Knight) * 2
            + material_value(PieceType::Pawn) * 8;

        let material_ratio =
            total_material.min(max_material).to_int() as f64 / max_material.to_int() as f64;
        let clock_penalty = (self.game.full_move_clock as f64 / 400.0).min(0.2);

        (material_ratio - clock_penalty).clamp(0.0, 1.0)
    }

    /// Score material based on its value and position on the board
    fn score_white_piece_positions(&self, ratio: f64) -> Score {
        let mut score = Score::default();

        for sq in self.game.white_occupied {
            let (piece, color) = self.game.piece_lookup(sq).unwrap();
            score += square_value(piece, sq, color, ratio);
        }

        score
    }

    /// Score material based on its value and position on the board
    fn score_black_piece_positions(&self, ratio: f64) -> Score {
        let mut score = Score::default();

        for sq in self.game.black_occupied {
            let (piece, color) = self.game.piece_lookup(sq).unwrap();
            score += square_value(piece, sq, color, ratio);
        }

        score
    }

    /// Scores king safety. Primarily based on whether the king has friendly pawns next to him.
    fn score_white_king_safety(&self) -> Score {
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
        Score::new(((white_pawn_area & self.game.white_pawns).popcnt() * 15) as i32)
    }

    /// Scores king safety. Primarily based on whether the king has friendly pawns next to him.
    fn score_black_king_safety(&self) -> Score {
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

        let black_king = self.game.black_kings.to_square();
        let black_pawn_area = calculate_pawn_area(&black_king);
        Score::new(((black_pawn_area & self.game.black_pawns).popcnt() * 15) as i32)
    }

    /// Scores the position castling rights
    fn score_white_castling_rights(&self) -> Score {
        let mut score = Score::default();
        let value = 2;

        if self.game.castling_rights.white_queenside() {
            score += value;
        }

        if self.game.castling_rights.white_kingside() {
            score += value;
        }

        score
    }

    /// Scores the position castling rights
    fn score_black_castling_rights(&self) -> Score {
        let mut score = Score::default();
        let value = 2;

        if self.game.castling_rights.black_queenside() {
            score += value;
        }

        if self.game.castling_rights.black_kingside() {
            score += value;
        }

        score
    }

    fn score_white_attackers(&self) -> Score {
        Score::new(((self.game.white_attacks & self.game.occupied).popcnt() * 10) as i32)
    }

    fn score_black_attackers(&self) -> Score {
        Score::new(((self.game.black_attacks & self.game.occupied).popcnt() * 10) as i32)
    }

    /// Score everything related to black's position
    fn score_black(&self, black_material: Score, ratio: f64) -> Score {
        black_material
            + self.score_black_piece_positions(ratio)
            + self.score_black_attackers()
            + self.score_black_king_safety()
            + self.score_black_castling_rights()
    }

    /// Score everything related to whites position
    fn score_white(&self, white_material: Score, ratio: f64) -> Score {
        white_material
            + self.score_white_piece_positions(ratio)
            + self.score_white_attackers()
            + self.score_white_king_safety()
            + self.score_white_castling_rights()
    }

    /// This is meant to be called on states other than InProgress. InProgress will return 0.0
    fn score_state(&self, for_color: PieceColor) -> Score {
        match self.game.state {
            State::Checkmate => match for_color {
                PieceColor::White => Score::MAX,
                PieceColor::Black => Score::MIN,
            },
            State::Stalemate => Score::default(),
            // TODO. Timing out should result in a win for the opponent if the opponent has
            // sufficent checkmating material
            State::Timeout => Score::default(),
            State::Repetition => Score::default(),
            _ => Score::default(),
        }
    }

    /// Grades the position for white
    pub fn grade_position(&mut self) -> Score {
        if self.game.state != State::InProgress {
            return self.score_state(PieceColor::White);
        }

        let white_material = self.score_white_material();
        let black_material = self.score_black_material();
        let ratio = self.midgame_to_lategame_ratio(white_material + black_material);

        self.score_white(white_material, ratio) - self.score_black(black_material, ratio)
    }

    /// Grades the position for the current player's turn
    pub fn grade_position_relative(&mut self) -> Score {
        if self.game.state != State::InProgress {
            return self.score_state(self.game.turn);
        }

        let white_material = self.score_white_material();
        let black_material = self.score_black_material();
        let ratio = self.midgame_to_lategame_ratio(white_material + black_material);

        self.score_white(white_material, ratio) + self.score_black(black_material, ratio)
    }
}
