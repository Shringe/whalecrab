use crate::{
    board::Board,
    movegen::{moves::Move, pieces::piece::populate_ray_piece},
    square::{Direction, Square},
};

use super::piece::Piece;

pub struct Bishop(pub Square);

impl Piece for Bishop {
    fn psuedo_legal_moves(&self, board: &mut Board) -> Vec<Move> {
        let directions = [
            Direction::NorthEast,
            Direction::SouthEast,
            Direction::NorthWest,
            Direction::SouthWest,
        ];

        populate_ray_piece(&self.0, &directions, board)
    }
}

#[cfg(test)]
mod tests {
    use crate::{board::PieceType, test_utils::format_pretty_list};

    use super::*;

    #[test]
    fn white_bishop_can_move_around() {
        let mut board = Board::default();

        for m in [
            Move::new(Square::G2, Square::G4, &board),
            Move::new(Square::G8, Square::F6, &board),
            Move::new(Square::F1, Square::G2, &board),
            Move::new(Square::F6, Square::G8, &board),
            Move::new(Square::G2, Square::C6, &board),
            Move::new(Square::G8, Square::F6, &board),
            Move::new(Square::C6, Square::G2, &board),
            Move::new(Square::F6, Square::G8, &board),
            Move::new(Square::G2, Square::F1, &board),
        ] {
            if board.determine_piece(m.from) == Some(PieceType::Bishop) {
                let moves = Bishop(m.from).psuedo_legal_moves(&mut board);
                assert!(
                    moves.contains(&m),
                    "The move {} not be found naturally! Available {}",
                    m,
                    format_pretty_list(&moves)
                );
            }
            board = m.make(&board);
        }
    }
}
