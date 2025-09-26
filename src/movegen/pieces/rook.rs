use crate::{
    board::Board,
    movegen::moves::Move,
    square::{Direction, Square},
};

use super::piece::Piece;

pub struct Rook(pub Square);

impl Piece for Rook {
    fn psuedo_legal_moves(&self, board: &mut Board) -> Vec<Move> {
        let directions = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];

        self.0.rays(&directions, board)
    }
}

#[cfg(test)]
mod tests {
    use crate::{board, test_utils::format_pretty_list};

    use super::*;

    #[test]
    fn white_rook_can_move_around() {
        let mut board = Board::default();

        for m in [
            Move::new(Square::A2, Square::A4, &board),
            Move::new(Square::G8, Square::F6, &board),
            Move::new(Square::A1, Square::A3, &board),
            Move::new(Square::F6, Square::G8, &board),
            Move::new(Square::A3, Square::H3, &board),
            Move::new(Square::G8, Square::F6, &board),
            Move::new(Square::H3, Square::A3, &board),
            Move::new(Square::F6, Square::G8, &board),
            Move::new(Square::A3, Square::A1, &board),
        ] {
            if board.determine_piece(m.from) == Some(board::PieceType::Rook) {
                let moves = Rook(m.from).psuedo_legal_moves(&mut board);
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
