use std::{io::Stdin, ops::MulAssign, str::FromStr, time::Duration};

use whalecrab_engine::{engine::Engine, score::Score};
use whalecrab_lib::{
    movegen::{moves::Move, pieces::piece::PieceColor},
    position::game::Game,
};

use crate::{command::UciCommand, log, received, sent};

const ID_NAME: &str = "whalecrab";
const ID_AUTHOR: &str = "Shringe";

#[derive(Debug, PartialEq)]
pub enum UciHandleAction {
    Quit,
    Continue,
}

/// Stores the state of the uci interface
pub struct UciInterface {
    pub engine: Engine,
    pub depth: u16,
    pub duration: Duration,
    /// The last score the engine came up with
    last_score: Score,
}

impl Default for UciInterface {
    fn default() -> Self {
        Self {
            engine: Engine::default(),
            depth: 20,
            #[cfg(debug_assertions)]
            duration: Duration::from_millis(30),
            #[cfg(not(debug_assertions))]
            duration: Duration::from_secs(3),
            last_score: Score::default(),
        }
    }
}

impl UciInterface {
    /// Runs the uci interface by watching stdin
    pub fn watch(&mut self, stdin: Stdin) {
        for line in stdin.lines() {
            let line = match line {
                Ok(line) => {
                    received!("{}", line);
                    line
                }
                Err(e) => {
                    log!("Failed to read stdin: {}", e);
                    continue;
                }
            };

            let cmd = match UciCommand::from_str(&line) {
                Ok(cmd) => cmd,
                Err(e) => {
                    log!("Failed to parse uci: {}", e);
                    continue;
                }
            };

            let (responses, action) = self.handle(cmd);
            for msg in &responses {
                sent!("{}", msg);
            }
            for msg in &responses {
                println!("{}", msg);
            }

            match action {
                UciHandleAction::Quit => break,
                UciHandleAction::Continue => continue,
            }
        }
    }

    /// Handles a single UciCommand. Returns a vector of responses and a UciHandleAction to
    /// describe things that must be handled by the caller.
    pub fn handle(&mut self, cmd: UciCommand) -> (Vec<String>, UciHandleAction) {
        let mut out = Vec::new();

        macro_rules! uci_send {
            ($($arg:tt)*) => {{
                let msg = format!($($arg)*);
                out.push(msg);
            }};
        }

        match cmd {
            UciCommand::UciNewGame => self.engine.with_new_game(Game::default()),
            UciCommand::Quit => return (out, UciHandleAction::Quit),
            UciCommand::IsReady => uci_send!("readyok"),

            UciCommand::Uci => {
                uci_send!("id name {ID_NAME}");
                uci_send!("id author {ID_AUTHOR}");
                uci_send!("option name Depth type spin default 20 min 0 max 200");
                uci_send!(
                    "option name MaxMoveTimeMs type spin default {} min 0 max {}",
                    Duration::from_secs(3).as_millis(),
                    Duration::from_hours(1).as_millis(),
                );
                uci_send!("uciok");
            }

            UciCommand::SetOption { name, value } => match name.to_lowercase().as_str() {
                "depth" => match value.parse::<u16>() {
                    Ok(depth) => {
                        log!("Setting depth to {}", depth);
                        self.depth = depth
                    }
                    Err(e) => {
                        log!("Failed to parse depth: {}", e);
                    }
                },
                "maxmovetimems" => match value.parse::<u64>() {
                    Ok(0) => {
                        log!("Move time limit disabled");
                        self.duration = Duration::MAX;
                    }
                    Ok(ms) => {
                        log!("Setting max move time to {}ms", ms);
                        self.duration = Duration::from_millis(ms);
                    }
                    Err(e) => log!("Failed to parse movetime: {}", e),
                },
                _ => {
                    log!("Unknown option: {}", name);
                }
            },

            UciCommand::Position { fen, uci_moves } => {
                log!("Received position: {fen}");

                let mut game = match Game::from_fen(&fen) {
                    Some(g) => g,
                    None => {
                        log!("Failed to parse fen {fen}. Defaulting to starting position");
                        Game::default()
                    }
                };

                // Play all moves in sequence
                log!("Playing moves: {:#?}", uci_moves);
                if !uci_moves.is_empty() {
                    for uci_move in uci_moves.split(' ') {
                        let move_to_play = match Move::from_uci(uci_move, &self.engine.game) {
                            Ok(m) => m,
                            Err(e) => {
                                log!("Failed to parse uci move '{}': {:?}", uci_move, e);
                                return (out, UciHandleAction::Continue);
                            }
                        };
                        log!("Playing move: {}", move_to_play);
                        game.play(&move_to_play);
                    }
                }
                log!("Final position FEN: {}", game.to_fen());
                log!("Game state: {:?}", game.state);

                self.engine.with_new_game(game);
            }

            UciCommand::Go {
                movetime,
                wtime,
                btime,
                ..
            } => {
                log!(
                    "Movetime {:?} || wtime {:?} || btime {:?}",
                    movetime,
                    wtime,
                    btime
                );
                let movetime = self.determine_movetime(movetime, wtime, btime);
                log!("Engine will target {:?} move duration", movetime);

                let result = self.engine.search(movetime, self.depth);
                log!("Search result");
                for line in result.to_string().lines() {
                    log!(" -- {}", line);
                }

                let best_move = match result.best_move {
                    Some(m) => m,
                    None => {
                        log!("No self.engine move found. Maybe the game is finished?");
                        log!("Game state: {:?}", self.engine.game.state);
                        return (out, UciHandleAction::Continue);
                    }
                };

                let best_move_uci = best_move.to_uci(&self.engine.game);
                log!("Fen before playing the move: {}", self.engine.game.to_fen());
                uci_send!("bestmove {}", best_move_uci);
                self.last_score = result.info.score;
            }
        }

        (out, UciHandleAction::Continue)
    }

    /// Decides how long the engine should spend searching for its move
    fn determine_movetime(
        &self,
        movetime: Option<Duration>,
        wtime: Option<Duration>,
        btime: Option<Duration>,
    ) -> Duration {
        if let Some(movetime) = movetime {
            // In "time per move" time controls, taking more than the specified movetime may cause the
            // engine to lose on time, so we allocate some overhead.
            return movetime.mul_f64(0.9);
        }

        let (ours, opponents) = match self.engine.game.turn {
            PieceColor::White => (wtime, btime),
            PieceColor::Black => (btime, wtime),
        };

        let Some(ours) = ours else {
            return self.duration;
        };

        let expected_moves_remaining = 30;

        let mut allocation = ours / expected_moves_remaining;

        // If we're losing, allocate more time
        let score = self.last_score.for_color(self.engine.game.turn);
        let score_multiplier = if score == 0 {
            1.0
        } else if score > 0 {
            0.8
        } else {
            (1.0 + (score / 500).to_int() as f64).min(2.0)
        };

        // If we are up on time, allocate more time
        let time_multiplier = if let Some(opponents) = opponents
            && opponents > Duration::ZERO
        {
            let ratio = ours.as_secs_f64() / opponents.as_secs_f64();
            ratio.sqrt().clamp(0.5, 2.0)
        } else {
            1.0
        };

        allocation = allocation.mul_f64(score_multiplier);
        allocation = allocation.mul_f64(time_multiplier);

        allocation.min(ours.mul_f64(0.9))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::uci;
    use std::time::Instant;
    use whalecrab_lib::square::Square;

    #[test]
    fn determine_movetime() {
        let uci = UciInterface::default();
        let remaining = Duration::from_secs(2);
        let min = Duration::from_millis(20);
        let max = Duration::from_millis(2000);
        let actual = uci.determine_movetime(None, Some(remaining), Some(remaining));
        assert!(actual > min);
        assert!(actual < max);
    }

    #[test]
    fn greeting() {
        let mut uci = UciInterface::default();
        let response = uci.handle(uci!("uci")).0;
        assert!(response.contains(&"uciok".to_string()));
    }

    #[test]
    fn new_game() {
        let mut uci = UciInterface::default();
        let _ = uci.handle(uci!("ucinewgame")).0;
        assert_eq!(uci.engine.game, Game::default());
        let response = uci.handle(uci!("isready")).0;
        assert!(response.contains(&"readyok".to_string()));
    }

    #[test]
    fn simple_game() {
        let mut uci = UciInterface::default();

        uci.handle(uci!("ucinewgame"));
        uci.handle(uci!("isready"));

        // Play a few moves and let the engine respond
        let movetime = Duration::from_millis(10);
        for _ in 0..10 {
            let start = Instant::now();
            let (responses, action) = uci.handle(uci!("go movetime {}", movetime.as_millis()));
            assert_eq!(action, UciHandleAction::Continue);
            let elapsed = start.elapsed();
            assert!(elapsed > movetime);
            assert!(elapsed < movetime * 2);

            let bestmove = responses.iter().find(|r| r.starts_with("bestmove"));
            let bestmove = bestmove.expect("Engine should return a bestmove");
            let mv = bestmove.strip_prefix("bestmove ").unwrap();

            // Mirror the engine
            let mv = format!(
                "{}{}",
                Square::from_str(&mv[..2]).unwrap().flip_side(),
                Square::from_str(&mv[2..]).unwrap().flip_side()
            );

            uci.handle(uci!("position startpos moves {}", mv));
        }
    }

    #[test]
    fn takes_queen_from_fen() {
        let fen = "k7/ppn5/8/8/3K1Q2/8/8/R7 b - - 0 1";
        let mut uci = UciInterface::default();
        uci.handle(uci!("position fen {fen}"));
        uci.handle(uci!("go movetime 100"));
        let actual = uci.engine.game.to_fen();
        let expected = "k7/pp6/4n3/8/3K1Q2/8/8/R7 w - - 1 2";
        assert_eq!(actual, expected);
    }
}
