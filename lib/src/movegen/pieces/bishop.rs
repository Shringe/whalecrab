use crate::{
    game::Game,
    movegen::moves::Move,
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
        self.ray_moves(&DIRECTIONS, game)
    }

    pub fn bishop_psuedo_legal_targets_fast(&self, game: &Game) -> PieceMoveInfo {
        self.rays(&DIRECTIONS, game)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bitboard::BitBoard, movegen::pieces::piece::PieceType, test_utils::format_pretty_list,
    };

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
            let m = Move::new(from, to, &game.position);
            let frombb = BitBoard::from_square(m.from(&game.position));
            if matches!(game.determine_piece(&frombb), Some((PieceType::Bishop, _))) {
                let moves = m.from(&game.position).bishop_psuedo_legal_moves(&mut game);
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
