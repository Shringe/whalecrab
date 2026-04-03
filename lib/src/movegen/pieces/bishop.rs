use crate::{
    position::game::Game,
    movegen::moves::{Move, targets_to_moves},
    square::{Direction, Square},
};

use super::piece::PieceMoveInfo;

const DIRECTIONS: [Direction; 4] = [
    Direction::NorthEast,
    Direction::SouthEast,
    Direction::NorthWest,
    Direction::SouthWest,
];

impl Square {
    pub fn bishop_psuedo_legal_moves(&self, game: &Game) -> Vec<Move> {
        targets_to_moves(self.bishop_psuedo_legal_targets(game).targets, *self, game)
    }

    pub fn bishop_psuedo_legal_targets(&self, game: &Game) -> PieceMoveInfo {
        self.rays(&DIRECTIONS, game)
    }
}

#[cfg(test)]
mod tests {
    use crate::{movegen::pieces::piece::PieceType, test_utils::format_pretty_list};

    use super::*;

    #[test]
    fn white_bishop_can_move_around() {
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
            if matches!(game.piece_lookup(from), Some((PieceType::Bishop, _))) {
                let moves = m.from(&game).bishop_psuedo_legal_moves(&game);
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
