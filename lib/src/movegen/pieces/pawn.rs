use crate::{
    bitboard::BitBoard,
    file::File,
    game::Game,
    movegen::{
        moves::{Move, targets_to_moves},
        pieces::piece::{PieceColor, PieceMoveInfo},
    },
    rank::Rank,
    square::Square,
};

impl Square {
    /// Generates all psuedo legal moves for a single pawn
    /// En_Passant is considered
    /// Promotion is considered (only for queen)
    /// King safety not considered
    pub fn pawn_psuedo_legal_moves(&self, game: &Game) -> Vec<Move> {
        targets_to_moves(self.pawn_psuedo_legal_targets(game).targets, *self, game)
    }

    pub fn pawn_psuedo_legal_targets(&self, game: &Game) -> PieceMoveInfo {
        let mut moveinfo = PieceMoveInfo::default();

        let sqbb = BitBoard::from_square(*self);
        let friendly = &game.turn;
        // let friendly = game
        //     .determine_color(&BitBoard::from_square(*self))
        //     .expect("Tried to move non existent pawn");

        let file = self.get_file();

        match friendly {
            PieceColor::White => {
                let oncebb = sqbb << BitBoard(8);
                if !game.occupied.has_square(&oncebb) {
                    moveinfo.targets |= oncebb;
                    if self.get_rank() == Rank::Second {
                        let twicebb = oncebb << BitBoard(8);
                        if !game.occupied.has_square(&twicebb) {
                            moveinfo.targets |= twicebb;
                        }
                    }
                }

                if file > File::A {
                    let uleft = sqbb << BitBoard(7);
                    moveinfo.attacks |= uleft;
                    if game.black_occupied.has_square(&uleft) {
                        moveinfo.targets |= uleft;
                    } else if let Some(target) = game.en_passant_target
                        && uleft == BitBoard::from_square(target)
                    {
                        moveinfo.targets |= uleft;
                    }
                }

                if file < File::H {
                    let uright = sqbb << BitBoard(9);
                    moveinfo.attacks |= uright;
                    if game.black_occupied.has_square(&uright) {
                        moveinfo.targets |= uright;
                    } else if let Some(target) = game.en_passant_target
                        && uright == BitBoard::from_square(target)
                    {
                        moveinfo.targets |= uright;
                    }
                }
            }

            PieceColor::Black => {
                let oncebb = sqbb >> BitBoard(8);
                if !game.occupied.has_square(&oncebb) {
                    moveinfo.targets |= oncebb;
                    if self.get_rank() == Rank::Seventh {
                        let twicebb = oncebb >> BitBoard(8);
                        if !game.occupied.has_square(&twicebb) {
                            moveinfo.targets |= twicebb;
                        }
                    }
                }

                if file > File::A {
                    let dleft = sqbb >> BitBoard(9);
                    moveinfo.attacks |= dleft;
                    if game.white_occupied.has_square(&dleft) {
                        moveinfo.targets |= dleft;
                    } else if let Some(target) = game.en_passant_target
                        && dleft == BitBoard::from_square(target)
                    {
                        moveinfo.targets |= dleft;
                    }
                }

                if file < File::H {
                    let dright = sqbb >> BitBoard(7);
                    moveinfo.attacks |= dright;
                    if game.white_occupied.has_square(&dright) {
                        moveinfo.targets |= dright;
                    } else if let Some(target) = game.en_passant_target
                        && dright == BitBoard::from_square(target)
                    {
                        moveinfo.targets |= dright;
                    }
                }
            }
        }

        moveinfo
    }
}

#[cfg(test)]
mod tests {
    use crate::{file::File, movegen::pieces::piece::PieceType, test_utils::format_pretty_list};

    use super::*;

    #[test]
    fn white_pawn_sees_black_target() {
        let mut game = Game::default();
        let looking_for = Move::Normal {
            from: Square::H4,
            to: Square::G5,
            capture: Some(PieceType::Pawn),
        };

        for m in [
            Move::Normal {
                from: Square::H2,
                to: Square::H4,
                capture: None,
            },
            Move::Normal {
                from: Square::G7,
                to: Square::G5,
                capture: None,
            },
        ] {
            game.play(&m);
        }

        assert_eq!(game.turn, PieceColor::White);
        let moves = looking_for.from(&game).pawn_psuedo_legal_moves(&game);
        assert!(
            moves.contains(&looking_for),
            "White pawn can't see target. {}",
            format_pretty_list(&moves)
        );
    }

    #[test]
    fn black_pawn_sees_white_target() {
        let mut game = Game::default();
        let looking_for = Move::Normal {
            from: Square::D5,
            to: Square::C4,
            capture: Some(PieceType::Pawn),
        };

        for m in [
            Move::CreateEnPassant {
                at: Square::C2.get_file(),
            },
            Move::Normal {
                from: Square::D7,
                to: Square::D5,
                capture: None,
            },
            Move::Normal {
                from: Square::H2,
                to: Square::H3,
                capture: None,
            },
        ] {
            game.play(&m);
        }

        assert_eq!(game.turn, PieceColor::Black);
        let moves = looking_for.from(&game).pawn_psuedo_legal_moves(&game);
        assert!(
            moves.contains(&looking_for),
            "Black pawn can't see target. Available moves: {:?}",
            moves
        );
    }

    #[test]
    fn white_pawn_sees_queen_promotion() {
        let mut game = Game::default();
        let looking_for = Move::Promotion {
            from: File::G,
            to: File::H,
            piece: PieceType::Queen,
            capture: Some(PieceType::Rook),
        };

        for (from, to) in [
            (Square::H2, Square::H4),
            (Square::G7, Square::G5),
            (Square::H4, Square::G5),
            (Square::H7, Square::H6),
            (Square::G5, Square::H6),
            (Square::F8, Square::G7),
            (Square::H6, Square::G7),
            (Square::E7, Square::E5),
        ] {
            let m = Move::infer(from, to, &game);
            game.play(&m);
        }

        assert_eq!(game.turn, PieceColor::White);
        let moves = looking_for.from(&game).pawn_psuedo_legal_moves(&game);
        assert!(
            moves.contains(&looking_for),
            "White pawn can't see target. Available moves: {:?}",
            moves
        );
    }
}
