use crate::{
    game::Game,
    movegen::{
        moves::{Move, targets_to_moves},
        pieces::piece::PieceMoveInfo,
    },
    square::{ALL_DIRECTIONS, Square},
};

impl Square {
    pub fn queen_psuedo_legal_moves(&self, game: &Game) -> Vec<Move> {
        targets_to_moves(
            self.queen_psuedo_legal_targets_fast(game).targets,
            *self,
            game,
        )
    }

    pub fn queen_psuedo_legal_targets_fast(&self, game: &Game) -> PieceMoveInfo {
        self.rays(&ALL_DIRECTIONS, game)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bitboard::BitBoard, movegen::pieces::piece::PieceType, test_utils::format_pretty_list,
    };

    use super::*;

    #[test]
    fn white_queen_can_move_like_rook() {
        let mut game = Game::default();

        for (from, to) in [
            (Square::A2, Square::A4),
            (Square::G8, Square::F6),
            (Square::A1, Square::A3),
            (Square::F6, Square::G8),
            (Square::A3, Square::H3),
            (Square::G8, Square::F6),
            (Square::H3, Square::A3),
            (Square::F6, Square::G8),
            (Square::A3, Square::A1),
        ] {
            let m = Move::infer(from, to, &game);
            let frombb = BitBoard::from_square(m.from(&game));
            if matches!(game.determine_piece(&frombb), Some((PieceType::Queen, _))) {
                let moves = m.from(&game).queen_psuedo_legal_moves(&game);
                assert!(
                    moves.contains(&m),
                    "The move {} not be found naturally! Available {}",
                    m,
                    format_pretty_list(&moves)
                );
            }
            game.play(&m);
        }
    }

    #[test]
    fn white_queen_can_move_like_bishop() {
        let mut game = Game::default();

        for (from, to) in [
            (Square::G2, Square::G4),
            (Square::G8, Square::F6),
            (Square::F1, Square::G2),
            (Square::F6, Square::G8),
            (Square::G2, Square::C6),
            (Square::G8, Square::F6),
            (Square::C6, Square::G2),
            (Square::F6, Square::G8),
            (Square::G2, Square::F1),
        ] {
            let m = Move::infer(from, to, &game);
            let frombb = BitBoard::from_square(m.from(&game));
            if matches!(game.determine_piece(&frombb), Some((PieceType::Queen, _))) {
                let moves = m.from(&game).queen_psuedo_legal_moves(&game);
                assert!(
                    moves.contains(&m),
                    "The move {} not be found naturally! Available {}",
                    m,
                    format_pretty_list(&moves)
                );
            }
            game.play(&m);
        }
    }
}
