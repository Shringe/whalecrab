use crate::{
    game::Game,
    movegen::moves::{Move, targets_to_moves},
    square::{Direction, Square},
};

use super::piece::PieceMoveInfo;

const DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::South,
    Direction::East,
    Direction::West,
];

impl Square {
    pub fn rook_psuedo_legal_moves(&self, game: &Game) -> Vec<Move> {
        targets_to_moves(self.rook_psuedo_legal_targets(game).targets, *self, game)
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
                let moves = m.from(&game).rook_psuedo_legal_moves(&game);
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
