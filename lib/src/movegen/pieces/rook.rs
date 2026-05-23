use crate::{
    bitboard::BitBoard,
    movegen::{
        moves::{Move, attacks_to_moves},
        pieces::piece::PieceType,
    },
    position::game::Game,
    square::{Direction, Square},
};

use super::piece::PieceMoveInfo;

pub const DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::South,
    Direction::East,
    Direction::West,
];

pub fn magic_rook_attacks(sq: Square, occupied: BitBoard) -> BitBoard {
    let rook = &magics::rooks::ROOKS[sq.index()];
    let key = (((occupied.to_int() & rook.mask).wrapping_mul(rook.magic))
        >> (magics::rooks::SHIFT as u64)) as usize;
    BitBoard::new(rook.attacks[key])
}

impl Square {
    pub fn rook_psuedo_legal_moves(&self, game: &Game) -> Vec<Move> {
        let color = game.piece_lookup(*self).map(|p| p.1).unwrap_or(game.turn);
        let kingbb = *game.get_pieces(&PieceType::King, &color.opponent());
        let blockers = game.occupied ^ kingbb;
        let attacks = magic_rook_attacks(*self, blockers);
        attacks_to_moves(attacks, *self, game)
    }

    pub fn rook_psuedo_legal_targets(&self, game: &Game) -> PieceMoveInfo {
        self.rays(&DIRECTIONS, game)
    }
}

#[cfg(test)]
mod tests {
    use crate::{movegen::pieces::piece::PieceType, test_utils::format_pretty_list};

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
            let m = Move::infer(from, to, &game);
            if matches!(game.piece_lookup(from), Some((PieceType::Rook, _))) {
                let moves = m.from(game.turn).rook_psuedo_legal_moves(&game);
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
