use std::sync::OnceLock;

use crate::transposition_table::TranspositionTable;
use whalecrab_lib::position::game::Game;

pub static TRANSPOSITION_TABLE_MEMORY_BUDGET_IN_KILOBYTES: OnceLock<usize> = OnceLock::new();

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Engine {
    /// Use self.with_new_game(game) instead of self.game = game if you want to replace this value
    pub game: Game,
    pub(crate) transposition_table: TranspositionTable,
}

impl Engine {
    pub fn from_game(game: Game) -> Engine {
        Engine {
            game,
            transposition_table: TranspositionTable::default(),
        }
    }

    /// Creates a position from fen and wraps the engine around it
    pub fn from_fen(fen: &str) -> Option<Engine> {
        Some(Engine::from_game(Game::from_fen(fen)?))
    }

    /// Resets any temporary engine values or caches and switches over to analyzing the new game.
    /// This should be used over replacing self.game manually
    pub fn with_new_game(&mut self, game: Game) {
        self.game = game
    }

    /// Clears caches that do not need bo be reset each game. This should only be called for
    /// testing and benchmarking purposes
    pub fn clear_persistant_cache(&mut self) {
        self.transposition_table.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::{score::Score, timers::infinite::Infinite};

    use super::*;
    use whalecrab_lib::{
        movegen::{moves::Move, pieces::piece::PieceType},
        position::game::State,
        square::Square,
    };

    /// Used for determining cache hit/miss
    fn time_grading(engine: &mut Engine) -> (Score, Duration) {
        let start_time = Instant::now();
        let result = engine.grade_position();
        let duration = start_time.elapsed();
        (result, duration)
    }

    #[track_caller]
    fn should_play(engine: &mut Engine, expected: Move, depth: u8) {
        let result = engine.search(Duration::MAX, depth);
        let actual = result.best.expect("The engine did not play a move");
        assert_eq!(actual, expected, "\n{}", result);
    }

    #[test]
    fn grading_should_not_mutate_position() {
        let mut engine = Engine::default();
        let before = engine.game.clone();
        let grade = engine.grade_position();
        let after = engine.game;
        println!("Score: {:?}", grade);
        assert_eq!(before, after);
    }

    #[test]
    fn starting_evaluation_is_balanced() {
        let mut engine = Engine::default();
        println!("{:?}", engine.game);
        let grade = engine.grade_position();
        println!("{:?}", engine.game);
        assert_eq!(grade, Score::default());
    }

    #[test]
    fn grade_position_is_deterministic() {
        let mut engine = Engine::default();
        let mut last_score = engine.grade_position();
        let mut last = engine.game.clone();
        for _ in 1..20 {
            let score = engine.grade_position();
            let game = engine.game.clone();
            assert_eq!(score, last_score);
            assert_eq!(game, last);
            last = game;
            last_score = score;
        }
    }

    #[test]
    fn black_always_takes_king() {
        let fen = "k6r/pp4r1/8/pp6/Qp6/pp6/7K/8 w - - 0 1";
        let mut engine = Engine::from_fen(fen).unwrap();
        let white_moves = engine.game.legal_moves();
        for m in white_moves {
            engine.game.play(&m);
            let result = engine.minimax(&Infinite, 0).best.unwrap();
            assert!(
                matches!(
                    result,
                    Move::Normal {
                        capture: Some(PieceType::King),
                        ..
                    }
                ),
                "Expected black to capture the king, got {:?}",
                result
            );
            engine.game.unplay(&m);
        }
    }

    #[test]
    fn white_always_checkmates() {
        let fen = "7k/8/8/8/8/8/5R2/K5R1 b - - 0 1";
        let mut engine = Engine::from_fen(fen).unwrap();
        let black_moves = engine.game.legal_moves();
        for m in black_moves {
            engine.game.play(&m);
            let looking_for = Move::infer(Square::F2, Square::H2, &engine.game);
            let result = engine.minimax(&Infinite, 1).best.unwrap();
            assert_eq!(result, looking_for);
            engine.game.unplay(&m);
        }
    }

    #[test]
    #[ignore]
    fn transportation_table_cache_hits() {
        let mut engine = Engine::default();

        let (initial_result, initial_duration) = time_grading(&mut engine);
        let min_speedup_factor = 1.2;

        for i in 1..100 {
            let (result, duration) = time_grading(&mut engine);
            assert_eq!(initial_result, result);

            let speedup_factor = initial_duration.as_nanos() as f64 / duration.as_nanos() as f64;

            assert!(
                speedup_factor >= min_speedup_factor,
                "Grading #{} was only {:.2}x faster than initial, but should be at least {:.1}x faster. \
            Initial: {:?}, Current: {:?}",
                i,
                speedup_factor,
                min_speedup_factor,
                initial_duration,
                duration
            );
        }
    }

    #[test]
    fn engine_moves_immutably() {
        let fen = "rnbqkbnr/pp1ppppp/2p5/8/4PP2/8/PPPP2PP/RNBQKBNR b KQkq - 0 2";
        let mut engine = Engine::from_fen(fen).unwrap();
        let before = engine.game.clone();
        let _ = engine.game.legal_moves();
        let _ = engine.minimax(&Infinite, 2).best;
        assert_eq!(before, engine.game);
    }

    #[test]
    fn engine_should_not_mutate_position() {
        let fen = "r1k2b1r/1p4p1/p1p4P/4B3/2p5/3P3P/NP2P1B1/2K2R2 w - - 0 29";
        let mut engine = Engine::from_fen(fen).unwrap();
        let before = engine.game.clone();
        let _ = engine.minimax(&Infinite, 3).best;
        let after = engine.game;
        assert_eq!(after, before);
    }

    #[test]
    fn should_have_moves_fen() {
        let fen = "rnbqkbnr/pp1ppppp/2p5/8/4PP2/8/PPPP2PP/RNBQKBNR b KQkq f3 0 2";
        let mut engine = Engine::from_fen(fen).unwrap();
        let moves = engine.game.legal_moves();
        let engine_move = engine.minimax(&Infinite, 2).best;
        assert!(!moves.is_empty());
        assert!(engine_move.is_some())
    }

    #[test]
    fn should_have_moves() {
        let mut engine = Engine::default();
        for (from, to) in [
            (Square::E2, Square::E4),
            (Square::C7, Square::C6),
            (Square::F2, Square::F4),
        ] {
            let m = Move::infer(from, to, &engine.game);
            engine.game.play(&m);
            let moves = engine.game.legal_moves();
            let engine_move = engine.minimax(&Infinite, 2).best;
            assert_eq!(engine.game.state, State::InProgress);
            assert!(!moves.is_empty());
            assert!(engine_move.is_some())
        }
    }

    #[test]
    fn should_find_mate_in_one() {
        let fen = "r3r1k1/pbP2p1p/6pb/8/P1Q5/3B1qP1/2R2P1P/1R4K1 b - - 1 37";
        let mut engine = Engine::from_fen(fen).unwrap();
        let expected = Move::infer(Square::F3, Square::H1, &engine.game);

        for n in 2..=4 {
            eprintln!("Depth: {}", n);
            let result = engine.minimax(&Infinite, n);
            eprintln!("{}", result);
            assert_eq!(result.best.unwrap(), expected);
        }
    }

    #[ignore = "TODO: fix test"]
    #[test]
    fn should_find_mate_in_3() {
        let fen = "kb5Q/p7/Pp6/1P6/4p3/4R3/4P1p1/6K1 w - - 0 1";
        let mut engine = Engine::from_fen(fen).unwrap();
        let expected = Move::infer(Square::E3, Square::H3, &engine.game);
        eprintln!("Possible moves: {:#?}", engine.game.legal_moves());
        should_play(&mut engine, expected, 7);
    }

    #[test]
    fn should_not_blunder_queen() {
        let fen = "r1b1k2r/pppp1ppp/2n1pn2/8/P1PPq3/2b1P2N/3NBPPP/1RBQ1RK1 b kq - 6 10";
        let mut engine = Engine::from_fen(fen).unwrap();
        let blunder = Move::Normal {
            from: Square::E4,
            to: Square::E3,
            capture: Some(PieceType::Pawn),
        };
        let recapture = Move::Normal {
            from: Square::F2,
            to: Square::E3,
            capture: Some(PieceType::Queen),
        };

        eprintln!("Score before playing: {}", engine.grade_position());

        assert!(engine.game.legal_moves().contains(&blunder));
        engine.game.play(&blunder);

        assert_ne!(engine.game.state, State::Checkmate);
        eprintln!("Score after playing: {}", engine.grade_position());
        assert!(
            engine
                .game
                .generate_all_psuedo_legal_moves()
                .contains(&recapture),
            "Engine doesn't see the opponent's recapture move as a possibilty"
        );
        assert!(
            engine.game.legal_moves().contains(&recapture),
            "Engine doesn't think the opponent's recapture move is legal"
        );

        engine.game.unplay(&blunder);

        let result = engine.search(Duration::MAX, 2);
        assert_ne!(
            result.best,
            Some(blunder),
            "The engine blunders its queen with Qxe3! Search: {:#?}",
            result
        );
    }

    #[test]
    fn should_take_hanging_pinning_bishop() {
        let fen = "rnbqk1nr/ppp2pp1/7p/3pp3/1b1PP3/8/PPPB1PPP/RN1QKBNR w KQkq - 0 1";
        let mut engine = Engine::from_fen(fen).unwrap();
        let expected = Move::infer(Square::D2, Square::B4, &engine.game);
        let result = engine.search(Duration::MAX, 2);
        assert_eq!(result.best, Some(expected));
    }
}
