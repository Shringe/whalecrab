use crate::{
    game::Game,
    movegen::moves::Move,
    square::{Direction, Square},
};

use super::piece::{Piece, PieceMoveInfo};

pub struct Bishop(pub Square);

impl Piece for Bishop {
    fn psuedo_legal_moves(&self, game: &mut Game) -> Vec<Move> {
        let directions = [
            Direction::NorthEast,
            Direction::SouthEast,
            Direction::NorthWest,
            Direction::SouthWest,
        ];

        self.0.ray_moves(&directions, game)
    }

    fn psuedo_legal_targets(&self, game: &Game) -> PieceMoveInfo {
        let directions = [
            Direction::NorthEast,
            Direction::SouthEast,
            Direction::NorthWest,
            Direction::SouthWest,
        ];

        self.0.rays(&directions, game)
    }
}

#[cfg(test)]
mod tests {
    use crate::{bitboard::BitBoard, board::PieceType, test_utils::format_pretty_list};

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
            let frombb = BitBoard::from_square(m.from);
            if matches!(game.determine_piece(&frombb), Some((PieceType::Bishop, _))) {
                let moves = Bishop(m.from).psuedo_legal_moves(&mut game);
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
