use crate::{
    bitboard::{BitBoard, EMPTY},
    board::{Board, Color, PieceType},
    movegen::moves::{get_targets, Move},
    square::Square,
};

pub trait Piece {
    /// Generates psuedo legal moves not considering king safety.
    fn psuedo_legal_moves(&self, board: &Board) -> Vec<Move>;

    /// Generates psuedo legal targets. Useful for highlighting squares in the TUI.
    fn psuedo_legal_targets(&self, board: &Board) -> Vec<Square> {
        let moves = self.psuedo_legal_moves(board);
        get_targets(moves)
    }

    /// Generates legal moves considering king safety.
    fn legal_moves(&self, board: &mut Board) -> Vec<Move> {
        let psuedo_legal = self.psuedo_legal_moves(&board);
        let mut legal = Vec::new();

        let attack_board = match board.turn {
            Color::White => {
                let mut attacking = board.white_attack_bitboard;
                for m in &psuedo_legal {
                    let tobb = BitBoard::from_square(m.to);
                    attacking |= tobb;
                }

                board.white_attack_bitboard = attacking;
                board.black_attack_bitboard
            }
            Color::Black => {
                let mut attacking = board.black_attack_bitboard;
                for m in &psuedo_legal {
                    let tobb = BitBoard::from_square(m.to);
                    attacking |= tobb;
                }

                board.black_attack_bitboard = attacking;
                board.white_attack_bitboard
            }
        };

        for m in psuedo_legal {
            let king_board = board.get_occupied_bitboard(&PieceType::King, &board.turn);
            let kingfrombb = BitBoard::from_square(m.from);
            let is_moving_king = king_board.has_square(&kingfrombb);

            if is_moving_king {
                let kingtobb = BitBoard::from_square(m.to);
                if attack_board.has_square(&kingtobb) {
                    continue;
                }
            }

            legal.push(m);
        }

        legal
    }

    /// Generates legal targets. Useful for highlighting squares in the TUI.
    fn legal_targets(&self, board: &mut Board) -> Vec<Square> {
        let moves = self.legal_moves(board);
        get_targets(moves)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_cant_blunder_king() {
        let fen = "1k6/1r6/8/8/8/8/8/K7 w - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        let psuedo_legal = board.generate_all_psuedo_legal_moves();
        let legal = board.generate_all_legal_moves();
        let legal = board.generate_all_legal_moves();

        let legal_looking_for = vec![Move::new(Square::A1, Square::A2, &board)];
        let psuedo_legal_looking_for = vec![
            Move::new(Square::A1, Square::A2, &board),
            Move::new(Square::A1, Square::B1, &board),
            Move::new(Square::A1, Square::B2, &board),
        ];

        assert_eq!(psuedo_legal, psuedo_legal_looking_for);
        assert_eq!(legal, legal_looking_for);
    }
}
