use crate::{
    board::Board,
    movegen::moves::{Move, MoveType},
    square::{Direction, Square},
};

use super::piece::Piece;

pub struct King(pub Square);

impl Piece for King {
    /// King safety not considered. Castling not yet implemented.
    fn psuedo_legal_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();
        let enemy = board.turn.opponent();

        for d in [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
            Direction::NorthEast,
            Direction::NorthWest,
            Direction::SouthEast,
            Direction::SouthWest,
        ] {
            if let Some(sq) = self.0.walk(&d) {
                if let Some(piece) = board.determine_color(sq) {
                    if piece == enemy {
                        moves.push(Move {
                            from: self.0,
                            to: sq,
                            variant: MoveType::Normal,
                        })
                    }
                } else {
                    moves.push(Move {
                        from: self.0,
                        to: sq,
                        variant: MoveType::Normal,
                    })
                }
            }
        }

        moves
    }
}

#[cfg(test)]
mod tests {
    use crate::{board::PieceType, test_utils::format_pretty_list};

    use super::*;

    #[test]
    fn double_bongcloud() {
        let mut board = Board::default();

        for m in [
            Move::new(Square::E2, Square::E4, &board),
            Move::new(Square::E7, Square::E5, &board),
            Move::new(Square::E1, Square::E2, &board),
            Move::new(Square::E8, Square::E7, &board),
            Move::new(Square::E2, Square::D3, &board),
            Move::new(Square::E7, Square::F6, &board),
        ] {
            if board.determine_piece(m.from) == Some(PieceType::King) {
                let moves = King(m.from).psuedo_legal_moves(&board);
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
