use crate::{
    bitboard::{BitBoard, EMPTY},
    position::game::Game,
    rank::Rank,
};

/// Checks that game meets the specified conditions. You can turn off specific validity checks by disabling flags
pub struct GameValidator {
    pub exactly_one_king_per_side: bool,
    pub opponent_must_not_be_in_check: bool,
    pub no_more_than_16_pieces_per_side: bool,
    pub no_more_than_eight_pawns_plus_promoted_pieces_per_side: bool,
    pub no_pawns_on_promotion_ranks: bool,
}

impl Default for GameValidator {
    /// Checks for every condition
    fn default() -> Self {
        Self {
            exactly_one_king_per_side: true,
            opponent_must_not_be_in_check: true,
            no_more_than_16_pieces_per_side: true,
            no_more_than_eight_pawns_plus_promoted_pieces_per_side: true,
            no_pawns_on_promotion_ranks: true,
        }
    }
}

impl GameValidator {
    /// Checks for no conditions; positions will always be considered valid
    pub fn empty() -> Self {
        Self {
            exactly_one_king_per_side: false,
            opponent_must_not_be_in_check: false,
            no_more_than_16_pieces_per_side: false,
            no_more_than_eight_pawns_plus_promoted_pieces_per_side: false,
            no_pawns_on_promotion_ranks: false,
        }
    }

    fn exactly_one_king_per_side(game: &Game) -> bool {
        game.white_kings.popcnt() == 1 && game.black_kings.popcnt() == 1
    }

    fn opponent_must_not_be_in_check(game: &Game) -> bool {
        game.is_in_check(game.turn.opponent())
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

    /// Determines if the game is legal
    #[rustfmt::skip]
    pub fn validate(&self, game: &Game) -> bool {
        (!self.exactly_one_king_per_side
            || Self::exactly_one_king_per_side(game))
        && (!self.opponent_must_not_be_in_check
            || Self::opponent_must_not_be_in_check(game))
        && (!self.no_more_than_16_pieces_per_side
            || Self::no_more_than_16_pieces_per_side(game))
        && (!self.no_more_than_eight_pawns_plus_promoted_pieces_per_side
            || Self::no_more_than_eight_pawns_plus_promoted_pieces_per_side(game))
        && (!self.no_pawns_on_promotion_ranks
            || Self::no_pawns_on_promotion_ranks(game))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn illegal() {
        let fen = "5r1k/5ppp/8/8/4PP2/1N1P1P2/QNRNP1P1/K1RB1P2 w - - 0 1";
        let game = Game::from_fen(fen).unwrap();
        assert!(GameValidator::empty().validate(&game));
        assert!(!GameValidator::default().validate(&game));
    }

    #[test]
    fn starting_position() {
        let game = Game::default();
        let validator = GameValidator::default();
        assert!(validator.validate(&game));
    }
}
