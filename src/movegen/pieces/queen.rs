use crate::{board::Board, movegen::moves::Move, square::Square};

use super::{bishop::Bishop, piece::Piece, rook::Rook};

pub struct Queen(pub Square);

impl Piece for Queen {
    fn psuedo_legal_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();
        moves.extend(Rook(self.0).psuedo_legal_moves(board));
        moves.extend(Bishop(self.0).psuedo_legal_moves(board));
        moves
    }
}

#[cfg(test)]
mod tests {
    use crate::{board::PieceType, test_utils::format_pretty_list};

    use super::*;

    #[test]
    fn white_queen_can_move_around_like_rook() {
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
            if board.determine_piece(m.from) == Some(PieceType::Queen) {
                let moves = Queen(m.from).psuedo_legal_moves(&board);
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

    #[test]
    fn white_queen_can_move_like_bishop() {
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
            if board.determine_piece(m.from) == Some(PieceType::Queen) {
                let moves = Queen(m.from).psuedo_legal_moves(&board);
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
