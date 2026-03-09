use std::{fmt, str::FromStr};

use crate::{
    bitboard::BitBoard,
    castling::{self, CastleSide},
    file::File,
    game::Game,
    movegen::pieces::piece::{PieceColor, PieceType},
    rank::Rank,
    square::Square,
};

/// Converts a vector of moves to a vector of targets
pub fn get_targets(moves: Vec<Move>, game: &Game) -> Vec<Square> {
    moves.into_iter().map(|m| m.to(game)).collect()
}

#[derive(PartialEq, Clone, Copy)]
pub enum Move {
    Normal {
        from: Square,
        to: Square,
        capture: Option<PieceType>,
    },
    CreateEnPassant {
        at: File,
    },
    CaptureEnPassant {
        from: File,
    },
    Promotion {
        from: File,
        to: File,
        piece: PieceType,
        capture: Option<PieceType>,
    },
    Castle {
        side: CastleSide,
    },
}

impl Move {
    /// Returns the destination square of the move. Consumes self
    pub fn to(self, game: &Game) -> Square {
        match self {
            Move::Normal { to, .. } => to,
            Move::CreateEnPassant { at } => match game.turn {
                PieceColor::White => Square::make_square(Rank::Fourth, at),
                PieceColor::Black => Square::make_square(Rank::Fifth, at),
            },
            Move::CaptureEnPassant { .. } => game.en_passant_target.expect(
                "A CaptureEnpassant move was created despite there being no en_passant target on the board",
            ),
            Move::Promotion { to, .. } => match game.turn {
                PieceColor::White => Square::make_square(Rank::Eighth, to),
                PieceColor::Black => Square::make_square(Rank::First, to),
            }
            Move::Castle { side } => match (game.turn, side) {
                (PieceColor::White, CastleSide::Queenside) => castling::WHITE_CASTLE_QUEENSIDE_KING_TO,
                (PieceColor::White, CastleSide::Kingside) => castling::WHITE_CASTLE_KINGSIDE_KING_TO,
                (PieceColor::Black, CastleSide::Queenside) => castling::BLACK_CASTLE_QUEENSIDE_KING_TO,
                (PieceColor::Black, CastleSide::Kingside) => castling::BLACK_CASTLE_KINGSIDE_KING_TO,
            }
        }
    }

    /// Returns the source square of the move. Consumes self
    pub fn from(self, game: &Game) -> Square {
        match self {
            Move::Normal { from, .. } => from,
            Move::CreateEnPassant { at } => match game.turn {
                PieceColor::White => Square::make_square(Rank::Second, at),
                PieceColor::Black => Square::make_square(Rank::Seventh, at),
            },
            Move::CaptureEnPassant { from } => match game.turn {
                PieceColor::White => Square::make_square(Rank::Fifth, from),
                PieceColor::Black => Square::make_square(Rank::Fourth, from),
            },
            Move::Promotion { from, .. } => match game.turn {
                PieceColor::White => Square::make_square(Rank::Seventh, from),
                PieceColor::Black => Square::make_square(Rank::Second, from),
            },
            Move::Castle { .. } => match game.turn {
                PieceColor::White => Square::E1,
                PieceColor::Black => Square::E8,
            },
        }
    }

    /// Returns true if the move captures a piece
    pub fn is_capture(&self) -> bool {
        match self {
            Move::Normal { capture, .. } => capture.is_some(),
            Move::Promotion { capture, .. } => capture.is_some(),
            _ => false,
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Move::Normal { from, to, capture } => {
                write!(f, "{} -> {}, Normal, Capturing: {:?}", from, to, capture)
            }
            Move::CreateEnPassant { at } => write!(f, "CreateEnPassant at {:?}", at),
            Move::CaptureEnPassant { from } => write!(f, "CaptureEnPassant from {:?}", from),
            Move::Promotion {
                from,
                to,
                piece,
                capture,
            } => {
                write!(
                    f,
                    "{:?} -> {:?}, Promoting to {:?}, Capturing: {:?}",
                    from, to, piece, capture
                )
            }
            Move::Castle { side } => write!(f, "Castle {:?}", side),
        }
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Move {
    /// Infers the type of move. This is likely already known during move generation, and in that
    /// case it is recommended to skip using this constructor.
    pub fn new(from: Square, to: Square, game: &Game) -> Self {
        match (&game.turn, from, to) {
            (PieceColor::White, Square::E1, Square::C1) if game.castling_rights.white_queenside => {
                Move::Castle {
                    side: CastleSide::Queenside,
                }
            }
            (PieceColor::White, Square::E1, Square::G1) if game.castling_rights.white_kingside => {
                Move::Castle {
                    side: CastleSide::Kingside,
                }
            }
            (PieceColor::Black, Square::E8, Square::C8) if game.castling_rights.black_queenside => {
                Move::Castle {
                    side: CastleSide::Queenside,
                }
            }
            (PieceColor::Black, Square::E8, Square::G8) if game.castling_rights.black_kingside => {
                Move::Castle {
                    side: CastleSide::Kingside,
                }
            }
            _ => {
                macro_rules! to_piece_type {
                    () => {
                        game.determine_piece(&BitBoard::from_square(to))
                            .map(|(t, _)| t)
                    };
                }

                let frombb = BitBoard::from_square(from);
                let (piece_type, piece_color) = game
                    .determine_piece(&frombb)
                    .expect("Tried to construct a move from a nonexistant piece");

                if piece_type == PieceType::Pawn {
                    if game.en_passant_target == Some(to) {
                        Move::CaptureEnPassant {
                            from: from.get_file(),
                        }
                    } else if let Some(once) = from.forward(&piece_color) {
                        if let Some(twice) = once.forward(&piece_color) {
                            if to == twice {
                                Move::CreateEnPassant {
                                    at: from.get_file(),
                                }
                            } else {
                                Move::Normal {
                                    from,
                                    to,
                                    capture: to_piece_type!(),
                                }
                            }
                        } else if once.get_rank() == piece_color.final_rank() {
                            Move::Promotion {
                                from: from.get_file(),
                                to: to.get_file(),
                                piece: PieceType::Queen,
                                capture: to_piece_type!(),
                            }
                        } else {
                            Move::Normal {
                                from,
                                to,
                                capture: to_piece_type!(),
                            }
                        }
                    } else {
                        Move::Normal {
                            from,
                            to,
                            capture: to_piece_type!(),
                        }
                    }
                } else {
                    Move::Normal {
                        from,
                        to,
                        capture: to_piece_type!(),
                    }
                }
            }
        }
    }

    /// Formats the move in uci notation, such as e2e4
    pub fn to_uci(&self, game: &Game) -> String {
        format!(
            "{}{}",
            self.from(game).to_string().to_lowercase(),
            self.to(game).to_string().to_lowercase()
        )
    }

    /// Returns a move from a uci string
    pub fn from_uci(uci: &str, game: &Game) -> Result<Self, ()> {
        Ok(Move::new(
            Square::from_str(&uci[..2])?,
            Square::from_str(&uci[2..])?,
            game,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{game::Game, test_utils::should_generate};

    #[test]
    fn should_be_promotion() {
        let fen = "5q2/6P1/8/8/8/6rr/RR6/KN4nk w - - 0 1";
        let game = Game::from_fen(fen).unwrap();
        let m = Move::new(Square::G7, Square::F8, &game);
        assert_eq!(
            m,
            Move::Promotion {
                from: File::G,
                to: File::F,
                piece: PieceType::Queen,
                capture: Some(PieceType::Queen),
            }
        );
    }

    #[test]
    fn to_uci() {
        let uci = "e2e4";
        let game = Game::default();
        let m = Move::Normal {
            from: Square::E2,
            to: Square::E4,
            capture: None,
        };

        assert_eq!(m.to_uci(&game), uci.to_owned());
    }

    #[test]
    fn from_uci() {
        let game = Game::default();
        let uci = "e2e4";
        let m = Move::CreateEnPassant { at: File::E };

        assert_eq!(Move::from_uci(uci, &game).unwrap(), m);
    }

    #[test]
    fn from_uci_capture() {
        let fen = "3qkbnr/1p3ppp/2n5/1ppbp3/8/r1pPBP1P/1P2P1P1/3QKBNR w Kk - 0 13";
        let mut game = Game::from_fen(fen).unwrap();

        let uci = "e3c5";
        let looking_for = Move::Normal {
            from: Square::E3,
            to: Square::C5,
            capture: Some(PieceType::Pawn),
        };

        let moves = game.generate_all_legal_moves();
        should_generate(&moves, &looking_for);
        assert_eq!(Move::from_uci(uci, &game).unwrap(), looking_for);
    }
}
