use std::{fmt, str::FromStr};

use crate::{
    bitboard::{BitBoard, EMPTY},
    file::File,
    movegen::pieces::piece::{PieceColor, PieceType},
    position::{
        castling::{self, CastleSide},
        game::{Game, State},
    },
    rank::Rank,
    square::{Square, SquareParseError},
    vectors::{UnsafeVec, Vector},
};

/// This assumes that the largest notation possible is something like "Nc3xd5+",
/// which is 7 bytes.
const MAX_SHORTHAND_NOTATION_EXPECTED_BYTES: usize = 7;

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

pub fn push_attacks_to_moves_with_occupied<V: Vector<Move>>(
    moves: &mut V,
    attacks: BitBoard,
    from: Square,
    game: &Game,
    enemy_occupied: BitBoard,
) {
    let walks = attacks & !game.occupied;
    for sq in walks {
        moves.push(Move::Normal {
            from,
            to: sq,
            capture: None,
        });
    }

    let captures = attacks & enemy_occupied;
    for sq in captures {
        moves.push(Move::Normal {
            from,
            to: sq,
            capture: Some(unsafe { game.piece_lookup(sq).unwrap_unchecked().0 }),
        });
    }
}

pub fn push_attacks_to_moves<V: Vector<Move>>(
    moves: &mut V,
    attacks: BitBoard,
    from: Square,
    game: &Game,
) {
    push_attacks_to_moves_with_occupied(
        moves,
        attacks,
        from,
        game,
        *game.get_occupied(&game.turn.opponent()),
    );
}

pub fn lazy_attacks_to_moves_with_occupied(
    attacks: BitBoard,
    from: Square,
    game: &Game,
    enemy_occupied: BitBoard,
) -> impl Iterator<Item = Move> {
    let walks = attacks & !game.occupied;
    let walk_moves = walks.map(move |sq| Move::Normal {
        from,
        to: sq,
        capture: None,
    });

    let captures = attacks & enemy_occupied;
    let capture_moves = captures.map(move |sq| Move::Normal {
        from,
        to: sq,
        capture: Some(unsafe { game.piece_lookup(sq).unwrap_unchecked().0 }),
    });

    walk_moves.chain(capture_moves)
}

pub fn lazy_attacks_to_moves(
    attacks: BitBoard,
    from: Square,
    game: &Game,
) -> impl Iterator<Item = Move> {
    let walks = attacks & !game.occupied;
    let walk_moves = walks.map(move |sq| Move::Normal {
        from,
        to: sq,
        capture: None,
    });

    let captures = attacks & *game.get_occupied(&game.turn.opponent());
    let capture_moves = captures.map(move |sq| Move::Normal {
        from,
        to: sq,
        capture: Some(unsafe { game.piece_lookup(sq).unwrap_unchecked().0 }),
    });

    walk_moves.chain(capture_moves)
}

// Converts a BitBoard of attacks into  vector of moves
pub fn attacks_to_moves(attacks: BitBoard, from: Square, game: &Game) -> Vec<Move> {
    let mut moves = UnsafeVec::with_capacity(attacks.popcnt() as usize);
    push_attacks_to_moves(&mut moves, attacks, from, game);
    moves.finish()
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
            Move::CaptureEnPassant { .. } => true,
            _ => false,
        }
    }

    /// Returns the captured piece if available
    pub fn capture(&self) -> Option<PieceType> {
        match self {
            Move::Normal { capture, .. } => *capture,
            Move::Promotion { capture, .. } => *capture,
            Move::CaptureEnPassant { .. } => Some(PieceType::Pawn),
            _ => None,
        }
    }

    /// Formats the move in uci notation, such as e2e4
    pub fn to_shorthand(self, game: &mut Game) -> String {
        match self {
            Move::Normal { from, to, capture } => {
                let mut out = String::with_capacity(MAX_SHORTHAND_NOTATION_EXPECTED_BYTES);

                let is_capture = capture.is_some();

                if let Some((piece, color)) = game.piece_lookup(from) {
                    if piece == PieceType::Pawn {
                        if is_capture {
                            out.push(from.get_file().notation());
                        }
                    } else {
                        out.push(piece.notation());
                        let attackers = game.attackers(to) & *game.get_pieces(&piece, &color);
                        if attackers.popcnt() > 1 {
                            let rank = from.get_rank();
                            let file = from.get_file();
                            let horizontal_attackers = attackers & rank.mask();
                            let vertical_attackers = attackers & file.mask();
                            if horizontal_attackers.popcnt() > 1 {
                                out.push(file.notation());
                            }
                            if vertical_attackers.popcnt() > 1 {
                                out.push(rank.notation());
                            }
                        }
                    }
                }

                if is_capture {
                    out.push('x');
                }

                out.push_str(&to.to_string().to_ascii_lowercase());

                game.play(&self);

                if game.state == State::Checkmate {
                    out.push('#');
                } else if game.is_in_check(game.turn) {
                    out.push('+');
                }

                game.unplay(&self);

                out
            }
            Move::CreateEnPassant { at } => {
                format!("{}{}", at.notation(), game.turn.create_en_passant_rank())
            }
            Move::CaptureEnPassant { from } => {
                format!("{}x{}",
                    from.notation(),
                    game.en_passant_target.expect(
                        "Tried to create shorthand notation for capturing an en_passant, but there was no en_passant_target available"
                    ).to_string().to_ascii_lowercase()
                )
            }
            Move::Promotion {
                from,
                to: _,
                piece,
                capture: None,
            } => {
                format!(
                    "{}{}={}",
                    from.notation(),
                    game.turn.final_rank(),
                    piece.notation()
                )
            }
            Move::Promotion {
                from,
                to,
                piece,
                capture: Some(_),
            } => {
                format!(
                    "{}x{}{}={}",
                    from.notation(),
                    to.notation(),
                    game.turn.final_rank(),
                    piece.notation()
                )
            }
            Move::Castle {
                side: CastleSide::Queenside,
            } => "O-O-O".to_string(),
            Move::Castle {
                side: CastleSide::Kingside,
            } => "O-O".to_string(),
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
    fn to_shorthand() {
        let fen = "2bqk2r/P1ppp1pp/8/8/5p2/8/PPPPPPPp/R3KBN1 w Qk - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let white_castles_queenside = Move::Castle {
            side: CastleSide::Queenside,
        };
        let white_promotes_to_queen = Move::Promotion {
            from: File::A,
            to: File::A,
            piece: PieceType::Queen,
            capture: None,
        };
        let black_castles_kingside = Move::Castle {
            side: CastleSide::Kingside,
        };
        let black_promotes_to_knight = Move::Promotion {
            from: File::H,
            to: File::G,
            piece: PieceType::Knight,
            capture: Some(PieceType::Knight),
        };
        let white_creates_en_passant = Move::CreateEnPassant { at: File::G };
        let black_captures_en_passant = Move::CaptureEnPassant { from: File::F };
        let black_moves_rook = Move::Normal {
            from: Square::F8,
            to: Square::F2,
            capture: Some(PieceType::Pawn),
        };

        assert_eq!(white_castles_queenside.to_shorthand(&mut game), "O-O-O");
        assert_eq!(white_promotes_to_queen.to_shorthand(&mut game), "a8=Q");

        game.play(&white_castles_queenside);
        assert_eq!(black_castles_kingside.to_shorthand(&mut game), "O-O");
        assert_eq!(black_promotes_to_knight.to_shorthand(&mut game), "hxg1=N");

        game.play(&black_castles_kingside);
        assert_eq!(white_creates_en_passant.to_shorthand(&mut game), "g4");

        game.play(&white_creates_en_passant);
        assert_eq!(black_captures_en_passant.to_shorthand(&mut game), "fxg3");

        game.play(&black_captures_en_passant);
        game.play(&white_promotes_to_queen);
        assert_eq!(black_moves_rook.to_shorthand(&mut game), "Rxf2");
    }

    #[test]
    fn to_shorthand_pawn_capture() {
        let fen = "k7/8/8/6p1/5R2/8/8/K7 b - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let m = Move::infer(Square::G5, Square::F4, &game);
        let expected = "gxf4";
        let actual = m.to_shorthand(&mut game);
        assert_ne!(actual, "xf4", "Pawn file is always included");
        assert_eq!(actual, expected);
    }

    #[test]
    fn to_shorthand_bishop_capture() {
        let fen = "kp6/pp6/8/4r3/8/6B1/PP6/KP6 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let m = Move::infer(Square::G3, Square::E5, &game);
        let expected = "Bxe5";
        let actual = m.to_shorthand(&mut game);
        assert_eq!(actual, expected);
    }

    #[test]
    fn to_shorthand_bishop_capture_with_ambiguous_rank() {
        let fen = "kp6/pp4B1/8/4r3/8/6B1/PP6/KP6 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let m = Move::infer(Square::G3, Square::E5, &game);
        let expected = "B3xe5";
        let actual = m.to_shorthand(&mut game);
        assert_ne!(actual, "Bxe5", "Missing Rank specifier");
        assert_ne!(actual, "B7xe5", "Wrong bishop");
        assert_ne!(actual, "Bgxe5", "Unnecessary File specifier");
        assert_eq!(actual, expected);
    }

    #[test]
    fn to_shorthand_bishop_capture_with_ambiguous_file() {
        let fen = "kp6/pp6/8/4r3/8/2B3B1/PP6/KP6 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let m = Move::infer(Square::C3, Square::E5, &game);
        let expected = "Bcxe5";
        let actual = m.to_shorthand(&mut game);
        assert_ne!(actual, "Bxe5", "Missing File specifier");
        assert_ne!(actual, "Bgxe5", "Wrong bishop");
        assert_ne!(actual, "B7xe5", "Unnecessary Rank specifier");
        assert_eq!(actual, expected);
    }

    #[test]
    fn to_shorthand_bishop_capture_with_ambiguous_file_and_rank() {
        let fen = "k7/pp4B1/8/4r3/8/2B3B1/PP6/K7 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let m = Move::infer(Square::G3, Square::E5, &game);
        let expected = "Bg3xe5";
        let actual = m.to_shorthand(&mut game);
        assert_ne!(actual, "Bxe5", "Missing File and Rank specifiers");
        assert_ne!(actual, "Bgxe5", "Missing Rank specifier");
        assert_ne!(actual, "B3xe5", "Missing File specifier");
        assert_ne!(actual, "B7xe5", "Wrong bishop");
        assert_ne!(actual, "Bcxe5", "Wrong bishop");
        assert_eq!(actual, expected);
    }

    #[test]
    fn to_shorthand_slightly_ambiguous_en_passant_capture() {
        let fen = "kr6/pp6/8/5p2/5pP1/8/PP6/KR6 b - g3 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let noob = Move::infer(Square::F5, Square::G4, &game);
        let pro = Move::infer(Square::F4, Square::G3, &game);
        assert_eq!(noob.to_shorthand(&mut game), "fxg4");
        assert_eq!(pro.to_shorthand(&mut game), "fxg3");
    }

    #[test]
    fn to_shorthand_bishop_captures_with_check() {
        let fen = "r1bqk1nr/pppp1ppp/2n5/2b1p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let m = Move::infer(Square::C4, Square::F7, &game);
        let expected = "Bxf7+";
        let actual = m.to_shorthand(&mut game);
        assert_ne!(actual, "Bxf7", "Missing check specifier");
        assert_eq!(actual, expected);
    }

    #[test]
    fn to_shorthand_queen_captures_with_checkmate() {
        let fen = "r1bqk1nr/pppp1ppp/2n5/2b1p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let m = Move::infer(Square::H5, Square::F7, &game);
        let expected = "Qxf7#";
        let actual = m.to_shorthand(&mut game);
        assert_ne!(actual, "Qxf7", "Missing checkmate specifier");
        assert!(
            !actual.contains('+'),
            "There should not be a check specifier"
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn to_shorthand_stalemate() {
        let fen = "8/8/8/8/8/4k3/5p2/5K2 b - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        let m = Move::infer(Square::E3, Square::F3, &game);
        let expected = "Kf3";
        let actual = m.to_shorthand(&mut game);
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_shorthand_notation_expected_bytes_is_actually_max() {
        let max = MAX_SHORTHAND_NOTATION_EXPECTED_BYTES;
        let large_notations = ["Bg3xe5#", "gxh1=Q#", "Nc3xd5+"];
        for notation in large_notations {
            let size = notation.len();
            assert!(
                size <= max,
                "Found a notation larger than {max} bytes: {notation}"
            );
        }
    }
}
