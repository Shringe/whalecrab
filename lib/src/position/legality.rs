use std::fmt::{self, Debug, Display};

use crate::{
    bitboard::{BitBoard, EMPTY},
    position::game::Game,
    rank::Rank,
};

fn exactly_one_king_per_side(game: &Game) -> bool {
    game.white_kings.popcnt() == 1 && game.black_kings.popcnt() == 1
}

fn opponent_must_not_be_in_check(game: &Game) -> bool {
    !game.is_in_check(game.turn.opponent())
}

fn no_more_than_16_pieces_per_side(game: &Game) -> bool {
    game.white_occupied.popcnt() <= 16 && game.black_occupied.popcnt() <= 16
}

fn no_more_than_eight_pawns_plus_promoted_pieces_per_side(game: &Game) -> bool {
    game.white_pawns.popcnt()
        + game
            .white_queens
            .popcnt()
            .saturating_sub(BitBoard::INITIAL_WHITE_QUEENS.popcnt())
        + game
            .white_rooks
            .popcnt()
            .saturating_sub(BitBoard::INITIAL_WHITE_ROOKS.popcnt())
        + game
            .white_bishops
            .popcnt()
            .saturating_sub(BitBoard::INITIAL_WHITE_BISHOPS.popcnt())
        + game
            .white_knights
            .popcnt()
            .saturating_sub(BitBoard::INITIAL_WHITE_KNIGHTS.popcnt())
        <= 8
        && game.black_pawns.popcnt()
            + game
                .black_queens
                .popcnt()
                .saturating_sub(BitBoard::INITIAL_BLACK_QUEENS.popcnt())
            + game
                .black_rooks
                .popcnt()
                .saturating_sub(BitBoard::INITIAL_BLACK_ROOKS.popcnt())
            + game
                .black_bishops
                .popcnt()
                .saturating_sub(BitBoard::INITIAL_BLACK_BISHOPS.popcnt())
            + game
                .black_knights
                .popcnt()
                .saturating_sub(BitBoard::INITIAL_BLACK_KNIGHTS.popcnt())
            <= 8
}

fn no_pawns_on_promotion_ranks(game: &Game) -> bool {
    (game.white_pawns & Rank::Eighth.mask()) | (game.black_pawns & Rank::First.mask()) == EMPTY
}

/// The set of rules used to determine position legality
#[derive(Debug, Clone, Copy)]
pub struct Ruleset {
    pub exactly_one_king_per_side: bool,
    pub opponent_must_not_be_in_check: bool,
    pub no_more_than_16_pieces_per_side: bool,
    pub no_more_than_eight_pawns_plus_promoted_pieces_per_side: bool,
    pub no_pawns_on_promotion_ranks: bool,
}

impl Default for Ruleset {
    fn default() -> Self {
        Self::complete()
    }
}

impl Display for Ruleset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.verdict())
    }
}

impl Ruleset {
    /// A complete set of laws
    pub fn complete() -> Self {
        Self {
            exactly_one_king_per_side: true,
            opponent_must_not_be_in_check: true,
            no_more_than_16_pieces_per_side: true,
            no_more_than_eight_pawns_plus_promoted_pieces_per_side: true,
            no_pawns_on_promotion_ranks: true,
        }
    }

    /// An empty set of laws
    pub fn empty() -> Self {
        Self {
            exactly_one_king_per_side: false,
            opponent_must_not_be_in_check: false,
            no_more_than_16_pieces_per_side: false,
            no_more_than_eight_pawns_plus_promoted_pieces_per_side: false,
            no_pawns_on_promotion_ranks: false,
        }
    }

    /// Returns `true` if a law was violated
    pub fn guilty(self) -> bool {
        self.exactly_one_king_per_side
            || self.opponent_must_not_be_in_check
            || self.no_more_than_16_pieces_per_side
            || self.no_more_than_eight_pawns_plus_promoted_pieces_per_side
            || self.no_pawns_on_promotion_ranks
    }

    /// Returns `true` if no laws were violated
    pub fn innocent(self) -> bool {
        !self.guilty()
    }

    /// Describes the verdict of the position
    pub fn verdict(self) -> String {
        let mut speech = String::new();

        if self.exactly_one_king_per_side {
            speech.push_str("There was not exactly one king per side!\n");
        }
        if self.opponent_must_not_be_in_check {
            speech.push_str("The opponent is in check!\n");
        }
        if self.no_more_than_16_pieces_per_side {
            speech.push_str("There are more than 16 pieces on either side!\n");
        }
        if self.no_more_than_eight_pawns_plus_promoted_pieces_per_side {
            speech.push_str("There are more than 8 pawns plus promoted pieces on either side!\n");
        }
        if self.no_pawns_on_promotion_ranks {
            speech.push_str("There are pawns on the promotion ranks!\n");
        }

        if speech.is_empty() {
            speech.push_str("Case dismissed!\n");
        }

        speech
    }

    /// Judges the game for legality, and returns the set of laws that the game has broken. Only
    /// the laws set on `self` will be enforced.
    pub fn judge(&self, game: &Game) -> Ruleset {
        Ruleset {
            exactly_one_king_per_side: self.exactly_one_king_per_side
                && !exactly_one_king_per_side(game),
            opponent_must_not_be_in_check: self.opponent_must_not_be_in_check
                && !opponent_must_not_be_in_check(game),
            no_more_than_16_pieces_per_side: self.no_more_than_16_pieces_per_side
                && !no_more_than_16_pieces_per_side(game),
            no_more_than_eight_pawns_plus_promoted_pieces_per_side: self
                .no_more_than_eight_pawns_plus_promoted_pieces_per_side
                && !no_more_than_eight_pawns_plus_promoted_pieces_per_side(game),
            no_pawns_on_promotion_ranks: self.no_pawns_on_promotion_ranks
                && !no_pawns_on_promotion_ranks(game),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_more_than_eighth_pawns_plus_promoted_pieces_per_side() {
        let fen = "5r1k/5ppp/8/8/4PP2/1NRP1P2/QNRNP1P1/K1RB1P2 w - - 0 1";
        let game = Game::from_fen(fen).unwrap();
        let law = Ruleset::complete();
        let verdict = law.judge(&game);
        assert!(verdict.guilty(), "{}", verdict);
    }

    #[test]
    fn starting_position_should_be_legal() {
        let game = Game::default();
        let law = Ruleset::complete();
        let verdict = law.judge(&game);
        assert!(verdict.innocent(), "{}", verdict);
    }
}
