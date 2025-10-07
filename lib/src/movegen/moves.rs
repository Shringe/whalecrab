use std::{fmt, str::FromStr};

use crate::{
    board::Board,
    castling::CastleSide,
    game::Game,
    movegen::pieces::piece::{Color, PieceType},
    square::Square,
};

#[derive(PartialEq, Debug, Clone)]
pub enum MoveType {
    Normal,
    Capture(PieceType),
    CreateEnPassant,
    CaptureEnPassant,
    Promotion(PieceType),
    Castle(CastleSide),
}

/// Converts a vector of moves to a vector of taargets
pub fn get_targets(moves: Vec<Move>) -> Vec<Square> {
    moves.into_iter().map(|m| m.to).collect()
}

#[derive(PartialEq, Clone)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub variant: MoveType,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}, {:?}", self.from, self.to, self.variant)
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Move {
    /// Infers the type of variant move. This is likely already known during move generation, and in that
    /// case it is recommended to skip using this contructor.
    pub fn new(from: Square, to: Square, board: &Board) -> Self {
        Self {
            from,
            to,
            variant: match (&board.turn, from, to) {
                (Color::White, Square::E1, Square::C1) if board.castling_rights.white_queenside => {
                    MoveType::Castle(CastleSide::Queenside)
                }
                (Color::White, Square::E1, Square::G1) if board.castling_rights.white_kingside => {
                    MoveType::Castle(CastleSide::Kingside)
                }
                (Color::Black, Square::E8, Square::C8) if board.castling_rights.black_queenside => {
                    MoveType::Castle(CastleSide::Queenside)
                }
                (Color::Black, Square::E8, Square::G8) if board.castling_rights.black_kingside => {
                    MoveType::Castle(CastleSide::Kingside)
                }
                _ => {
                    if let Some(enemy) = board.determine_piece(to) {
                        MoveType::Capture(enemy)
                    } else if board.determine_piece(from) == Some(PieceType::Pawn) {
                        let color = board.determine_color(from).unwrap();
                        if Some(to) == board.en_passant_target {
                            MoveType::CaptureEnPassant
                        } else if let Some(once) = from.forward(&color) {
                            if let Some(twice) = once.forward(&color) {
                                if to == twice {
                                    MoveType::CreateEnPassant
                                } else {
                                    MoveType::Normal
                                }
                            } else if once.get_rank() == color.final_rank() {
                                MoveType::Promotion(PieceType::Queen)
                            } else if let Some(enemy) = board.determine_piece(to) {
                                MoveType::Capture(enemy)
                            } else {
                                MoveType::Normal
                            }
                        } else {
                            MoveType::Normal
                        }
                    } else {
                        MoveType::Normal
                    }
                }
            },
        }
    }

    /// Formats the move in uci notation, such as e2e4
    pub fn to_uci(&self) -> String {
        format!(
            "{}{}",
            self.from.to_string().to_lowercase(),
            self.to.to_string().to_lowercase()
        )
    }

    /// Returns a move from a uci string
    pub fn from_uci(uci: &str, position: &Board) -> Result<Self, ()> {
        Ok(Move::new(
            Square::from_str(&uci[..2])?,
            Square::from_str(&uci[2..])?,
            position,
        ))
    }
}

impl Game {
    /// Plays a move on the board, updating the position and engine values
    /// Deprecated, but exists for compatibility
    pub fn play(&mut self, m: &Move) {
        m.play(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;

    #[test]
    fn to_uci() {
        let uci = "e2e4";
        let m = Move {
            from: Square::E2,
            to: Square::E4,
            variant: MoveType::Normal,
        };

        assert_eq!(m.to_uci(), uci.to_owned());
    }

    #[test]
    fn from_uci() {
        let game = Game::default();
        let uci = "e2e4";
        let m = Move {
            from: Square::E2,
            to: Square::E4,
            variant: MoveType::CreateEnPassant,
        };

        assert_eq!(Move::from_uci(uci, &game.position).unwrap(), m);
    }
}
