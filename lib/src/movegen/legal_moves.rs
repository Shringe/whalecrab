use crate::{
    bitboard::{BitBoard, EMPTY},
    movegen::{moves::Move, pieces::piece::PieceType},
    position::game::Game,
    square::Square,
};

pub struct LegalMovesFilter<'a> {
    game: &'a Game,
    king: Square,
    kingbb: BitBoard,
    king_attackers: BitBoard,
    attack_board: BitBoard,
    checks: BitBoard,
}

impl<'a> LegalMovesFilter<'a> {
    pub fn new(game: &'a Game) -> Self {
        let enemy = game.turn.opponent();
        Self {
            game,
            king: (*game.get_pieces(&PieceType::King, &game.turn)).to_square(),
            kingbb: *game.get_pieces(&PieceType::King, &game.turn),
            king_attackers: game
                .attackers((*game.get_pieces(&PieceType::King, &game.turn)).to_square()),
            attack_board: *game.get_attacks(&enemy),
            checks: *game.get_check_rays(&enemy),
        }
    }

    pub fn check(&self, m: Move) -> bool {
        let from = m.from(self.game.turn);
        let to = m.to(self.game);
        let frombb = BitBoard::from_square(from);
        let tobb = BitBoard::from_square(to);

        if !self.check_special(m, from, frombb, to) {
            return false;
        }

        let is_moving_king = self.kingbb.has_square(frombb);

        // Handle being in check
        match self.king_attackers.popcnt() {
            1 => {
                let attacker = self.king_attackers.to_square();
                let attacking_piece = self.game.piece_lookup(attacker).unwrap().0;

                let is_blocking = !is_moving_king
                    && attacking_piece.is_ray_piece()
                    && attacker.path_to(self.king)
                        & attacking_piece
                            .psuedo_legal_targets_fast(self.game, &attacker)
                            .targets
                        & tobb
                        != EMPTY;

                let is_capturing_attacking_piece =
                    m.is_capture() && self.king_attackers.has_square(tobb);

                if !(is_moving_king || is_capturing_attacking_piece || is_blocking) {
                    return false;
                }
            }
            2 => {
                if !is_moving_king {
                    return false;
                }
            }
            _ => {}
        }

        if is_moving_king {
            // Prevent moving into check
            if self.attack_board.has_square(tobb) {
                return false;
            }
        } else {
            // Prevent moving a pinned piece
            if self.checks.has_square(frombb) {
                // TODO: there are surely faster ways to do this than computing `self.checkers` per pinned move,
                // such as computing this once per pinned piece in `LegalMovesFilter::new` or tracking
                // pinning pieces in `self.game`
                let (checking_piecebb, check_ray) = self
                    .game
                    .checkers(frombb)
                    .expect("Could not find the piece populating `self.checks`");
                // Allow capturing the checking piece & moving within the check ray
                if tobb != checking_piecebb
                    && !(check_ray.has_square(frombb) && check_ray.has_square(tobb))
                {
                    return false;
                }
            }
        }

        true
    }

    fn check_special(&self, m: Move, from: Square, frombb: BitBoard, to: Square) -> bool {
        if let Move::CaptureEnPassant { .. } = m {
            let pawn_rank = from.get_rank();
            let king_rank = self.king.get_rank();

            if pawn_rank != king_rank {
                return true;
            }

            let remaining_row =
                self.game.occupied ^ frombb ^ to.get_file().mask() & pawn_rank.mask();

            if remaining_row.popcnt() < 2 {
                return true;
            }

            // Ensure that the en_passant_capture does not leave an enemy horizontal ray
            // piece staring at our king
            let mut was_king_or_horizontal_ray = false;
            for sq in remaining_row {
                let (piece, color) = unsafe { self.game.piece_lookup(sq).unwrap_unchecked() };
                let is_king_or_horizontal_ray = (color == self.game.turn
                    && piece == PieceType::King)
                    || (color != self.game.turn
                        && (piece == PieceType::Rook || piece == PieceType::Queen));

                if is_king_or_horizontal_ray && was_king_or_horizontal_ray {
                    return false;
                }

                was_king_or_horizontal_ray = is_king_or_horizontal_ray;
            }

            return true;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::position::game::Game;

    #[test]
    fn pawn_recapture_through_queen_ray_should_be_legal() {
        let fen = "r1b1k2r/pppp1ppp/2n1pn2/8/P1PP4/2b1q2N/3NBPPP/1RBQ1RK1 w kq - 0 11";
        let game = Game::from_fen(fen).unwrap();
        let lmf = LegalMovesFilter::new(&game);
        let m = Move::Normal {
            from: Square::F2,
            to: Square::E3,
            capture: Some(PieceType::Queen),
        };
        assert!(!game.is_in_check(game.turn));
        assert!(!game.is_in_check(game.turn.opponent()));
        assert!(lmf.check(m));
    }

    #[test]
    fn pinned_piece_can_move_across_check_ray() {
        let fen = "8/k7/8/8/8/BBB5/K1R1q3/BBB5 w - - 0 1";
        let game = Game::from_fen(fen).unwrap();
        let left = Move::infer(Square::C2, Square::B2, &game);
        let right = Move::infer(Square::C2, Square::D2, &game);
        let capture_attacker = Move::infer(Square::C2, Square::E2, &game);
        let lmf = LegalMovesFilter::new(&game);
        assert!(lmf.check(left));
        assert!(lmf.check(right));
        assert!(lmf.check(capture_attacker));
    }
}
