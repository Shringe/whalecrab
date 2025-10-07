use crate::{
    game::Game,
    movegen::moves::Move,
    square::{Direction, Square},
};

use super::piece::Piece;

pub struct Rook(pub Square);

impl Piece for Rook {
    fn psuedo_legal_moves(&self, game: &mut Game) -> Vec<Move> {
        let directions = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];

        self.0.ray_moves(&directions, game)
    }
}

#[cfg(test)]
mod tests {
    use crate::{bitboard::BitBoard, board::PieceType, test_utils::format_pretty_list};

    use super::*;

    #[test]
    fn white_rook_can_move_around() {
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
            let m = Move::new(from, to, &game.position);
            let frombb = BitBoard::from_square(m.from);
            if matches!(game.determine_piece(&frombb), Some((PieceType::Rook, _))) {
                let moves = Rook(m.from).psuedo_legal_moves(&mut game);
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
