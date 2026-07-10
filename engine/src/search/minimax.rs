use crate::engine::Engine;
use crate::{move_result::SearchResult, timers::MoveTimer};

/// Plays a move, gets the score from the given method, and then unplays the move and returns that
/// score. Also does expensive validity checks in debug builds.
#[macro_export]
macro_rules! search_move {
    ($self:expr, $move:expr, $method:ident($($args:expr),*)) => {{
        #[cfg(debug_assertions)]
        let before = $self.game.clone();

        $self.game.play(&$move);

        #[cfg(debug_assertions)]
        let during = $self.game.clone();

        let score = $self.$method($($args),*);
        $self.game.unplay($move);

        #[cfg(debug_assertions)]
        assert_eq!(
            $self.game, before,
            "State changed after playing and unplaying {}\n  Before: {:?}\n  During: {:?}\n   After: {:?}\n",
            $move, before, during, $self.game
        );

        score
    }};
}

impl Engine {
    /// Continues searching at the given depth until the search finishes or the timer is over.
    ///
    /// Note: this now uses Negamax under the hood as of 1.6.0.
    /// This function will probably be deprecated in the future.
    pub fn minimax<T: MoveTimer>(&mut self, timer: &T, depth: u8) -> SearchResult {
        self.negamax(timer, depth)
    }
}

#[cfg(test)]
mod tests {
    use whalecrab_lib::{movegen::moves::Move, square::Square};

    use crate::timers::infinite::Infinite;

    use super::*;

    #[test]
    fn minimax_engine_takes_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3";
        let mut engine = Engine::from_fen(starting).unwrap();
        let looking_for = Move::infer(Square::C1, Square::G5, &engine.game);
        let result = engine
            .minimax(&Infinite, 2)
            .best
            .expect("No moves found");
        println!("State: {:?}", engine.game.state);
        assert_eq!(result, looking_for);
    }

    #[test]
    fn minimax_engine_saves_queen() {
        let starting = "rnb1kbnr/pppp1ppp/8/4p1q1/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 1 3";
        let mut engine = Engine::from_fen(starting).unwrap();
        let black_queens_before = engine.game.black_queens.popcnt();
        let result = engine
            .minimax(&Infinite, 2)
            .best
            .expect("No moves found");
        engine.game.play(&result);
        assert_eq!(black_queens_before, engine.game.black_queens.popcnt());
    }
}
