use std::{fmt, str::FromStr};

use crate::{
    bitboard::{BitBoard, EMPTY},
    file::File,
    movegen::pieces::piece::{PieceColor, PieceType},
    position::castling::{self, CastleSide},
    position::game::Game,
    rank::Rank,
    square::{Square, SquareParseError},
};

/// Converts a vector of moves to a vector of targets
pub fn moves_to_targets_vec(moves: &[Move], game: &Game) -> Vec<Square> {
    moves.iter().map(|m| m.to(game)).collect()
}

/// Converts a vector of moves into a BitBoard of targets
pub fn moves_to_targets(moves: &[Move], game: &Game) -> BitBoard {
    let mut out = EMPTY;
    moves.iter().for_each(|m| out.set(m.to(game)));
    out
}

pub fn lazy_attacks_to_moves(
    attacks: BitBoard,
    from: Square,
    game: &Game,
) -> impl Iterator<Item = Move> {
    let enemy_color = game.turn.opponent();
    attacks
        .into_iter()
        .filter_map(move |sq| match game.piece_lookup(sq) {
            None => Some(Move::Normal {
                from,
                to: sq,
                capture: None,
            }),
            Some((piece, color)) if color == enemy_color => Some(Move::Normal {
                from,
                to: sq,
                capture: Some(piece),
            }),
            Some(_) => None,
        })
}

// Converts a BitBoard of attacks into  vector of moves
pub fn attacks_to_moves(attacks: BitBoard, from: Square, game: &Game) -> Vec<Move> {
    let mut moves = Vec::with_capacity(attacks.popcnt() as usize);
    for m in lazy_attacks_to_moves(attacks, from, game) {
        moves.push(m);
    }
    moves
}

/// Converts a BitBoard of targets into a vector of moves
pub fn targets_to_moves(targets: BitBoard, from: Square, game: &Game) -> Vec<Move> {
    let mut moves = Vec::with_capacity(targets.popcnt() as usize);

    for sq in targets {
        let m = Move::infer(from, sq, game);
        moves.push(m);
    }

    moves
}

/// A Vec that does not reallocate capacity if necessary. Use this type only if the upper bound of
/// capacity is fully known at runtime.
/// This type should not be dropped without `UnsafeVec::finish()` being called first.
#[derive(Debug)]
pub struct UnsafeVec<T> {
    list: Vec<T>,
    counter: usize,
}

impl<T> UnsafeVec<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            list: Vec::with_capacity(capacity),
            counter: 0,
        }
    }

    /// # Safety
    /// Don't push more items than the capacity
    pub unsafe fn push_unchecked(&mut self, item: T) {
        debug_assert!(
            self.counter < self.list.capacity(),
            "Tried to push too many items to an UnsafeVec! Index: {:?}, Capacity: {:?}",
            self.counter,
            self.list.capacity()
        );
        debug_assert_ne!(self.counter, usize::MAX);
        unsafe {
            self.list.as_mut_ptr().add(self.counter).write(item);
            self.counter = self.counter.unchecked_add(1);
        }
    }

    pub fn finish(mut self) -> Vec<T> {
        unsafe { self.list.set_len(self.counter) };
        self.list
    }
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
    /// Infers the type of move from only the starting and destination square
    pub fn infer(from: Square, to: Square, game: &Game) -> Self {
        match (&game.turn, from, to) {
            (PieceColor::White, Square::E1, Square::C1)
                if game.castling_rights.white_queenside() =>
            {
                Move::Castle {
                    side: CastleSide::Queenside,
                }
            }
            (PieceColor::White, Square::E1, Square::G1)
                if game.castling_rights.white_kingside() =>
            {
                Move::Castle {
                    side: CastleSide::Kingside,
                }
            }
            (PieceColor::Black, Square::E8, Square::C8)
                if game.castling_rights.black_queenside() =>
            {
                Move::Castle {
                    side: CastleSide::Queenside,
                }
            }
            (PieceColor::Black, Square::E8, Square::G8)
                if game.castling_rights.black_kingside() =>
            {
                Move::Castle {
                    side: CastleSide::Kingside,
                }
            }
            _ => {
                macro_rules! to_piece_type {
                    () => {
                        game.piece_lookup(to).map(|(t, _)| t)
                    };
                }

                let (piece_type, piece_color) = game
                    .piece_lookup(from)
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

    /// Returns the destination square of the move. Consumes self
    pub fn to(self, game: &Game) -> Square {
        match self {
            Move::Normal { to, .. } => to,
            Move::CreateEnPassant { at } => match game.turn {
                PieceColor::White => Square::make_square(Rank::Fourth, at),
                PieceColor::Black => Square::make_square(Rank::Fifth, at),
            },
            Move::CaptureEnPassant { .. } => game.en_passant_target.unwrap_or(Square::E1),
            Move::Promotion { to, .. } => match game.turn {
                PieceColor::White => Square::make_square(Rank::Eighth, to),
                PieceColor::Black => Square::make_square(Rank::First, to),
            },
            Move::Castle { side } => match (game.turn, side) {
                (PieceColor::White, CastleSide::Queenside) => {
                    castling::WHITE_CASTLE_QUEENSIDE_KING_TO
                }
                (PieceColor::White, CastleSide::Kingside) => {
                    castling::WHITE_CASTLE_KINGSIDE_KING_TO
                }
                (PieceColor::Black, CastleSide::Queenside) => {
                    castling::BLACK_CASTLE_QUEENSIDE_KING_TO
                }
                (PieceColor::Black, CastleSide::Kingside) => {
                    castling::BLACK_CASTLE_KINGSIDE_KING_TO
                }
            },
        }
    }

    /// Returns the source square of the move. Consumes self
    pub fn from(self, color: PieceColor) -> Square {
        match self {
            Move::Normal { from, .. } => from,
            Move::CreateEnPassant { at } => match color {
                PieceColor::White => Square::make_square(Rank::Second, at),
                PieceColor::Black => Square::make_square(Rank::Seventh, at),
            },
            Move::CaptureEnPassant { from } => match color {
                PieceColor::White => Square::make_square(Rank::Fifth, from),
                PieceColor::Black => Square::make_square(Rank::Fourth, from),
            },
            Move::Promotion { from, .. } => match color {
                PieceColor::White => Square::make_square(Rank::Seventh, from),
                PieceColor::Black => Square::make_square(Rank::Second, from),
            },
            Move::Castle { .. } => match color {
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

    /// Formats the move in uci notation, such as e2e4
    pub fn to_uci(&self, game: &Game) -> String {
        format!(
            "{}{}",
            self.from(game.turn).to_string().to_lowercase(),
            self.to(game).to_string().to_lowercase()
        )
    }

    /// Returns a move from a uci string
    pub fn from_uci(uci: &str, game: &Game) -> Result<Self, SquareParseError> {
        Ok(Move::infer(
            Square::from_str(&uci[..2])?,
            Square::from_str(&uci[2..])?,
            game,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{position::game::Game, test_utils::should_generate};

    #[test]
    fn should_be_promotion() {
        let fen = "5q2/6P1/8/8/8/6rr/RR6/KN4nk w - - 0 1";
        let game = Game::from_fen(fen).unwrap();
        let m = Move::infer(Square::G7, Square::F8, &game);
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

        let moves = game.legal_moves();
        should_generate(&moves, &looking_for);
        assert_eq!(Move::from_uci(uci, &game).unwrap(), looking_for);
    }

    #[test]
    fn unsafe_vec() {
        let mut uv = UnsafeVec::<usize>::with_capacity(10);
        unsafe {
            uv.push_unchecked(5);
            uv.push_unchecked(10);
        }

        let ptr_before = uv.list.as_ptr();
        let v = uv.finish();
        let ptr_after = v.as_ptr();
        assert_eq!(ptr_before, ptr_after);

        let expected = vec![5, 10];
        let actual = v;
        assert_eq!(actual, expected);
    }
}
