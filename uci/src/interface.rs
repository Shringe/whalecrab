use std::{io::Stdin, str::FromStr, time::Duration};

use whalecrab_engine::engine::Engine;
use whalecrab_lib::{
    game::Game,
    movegen::{moves::Move, pieces::piece::PieceColor},
};

use crate::{command::UciCommand, log};

const ID_NAME: &str = "whalecrab";
const ID_AUTHOR: &str = "Shringe";

pub enum UciHandleAction {
    Quit,
    Continue,
}

/// Stores the state of the uci interface
pub struct UciInterface {
    pub engine: Engine,
    pub depth: u16,
    pub duration: Duration,
}

impl Default for UciInterface {
    fn default() -> Self {
        Self {
            engine: Engine::default(),
            depth: 20,
            duration: Duration::from_mins(1),
        }
    }
}

impl UciInterface {
    /// Runs the uci interface by watching stdin
    pub fn watch(&mut self, stdin: Stdin) {
        for line in stdin.lines() {
            let line = match line {
                Ok(line) => {
                    log!("Recieved: {}", line);
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

            let (responses, action) = self.handle(&cmd);
            for msg in &responses {
                log!("Sent: {}", msg);
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
    pub fn handle(&mut self, cmd: &UciCommand) -> (Vec<String>, UciHandleAction) {
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
                    Duration::from_mins(1).as_millis(),
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

            UciCommand::Position { uci_moves } => {
                let engine = &mut self.engine;

                // Reset to starting position
                engine.with_new_game(Game::default());

                // Play all moves in sequence
                log!("{:#?}", uci_moves);
                for uci_move in uci_moves.split(' ') {
                    let move_to_play = match Move::from_uci(uci_move, &engine.game) {
                        Ok(m) => m,
                        Err(e) => {
                            log!("Failed to parse uci move '{}': {:?}", uci_move, e);
                            return (out, UciHandleAction::Continue);
                        }
                    };
                    log!("Playing move: {}", move_to_play);
                    engine.game.play(&move_to_play);
                }
                log!("Final position FEN: {}", engine.game.to_fen());
                log!("Game state: {:?}", engine.game.state);
            }

            UciCommand::Go {
                movetime: _,
                wtime,
                btime,
                winc,
                binc,
            } => {
                let (time, inc) = match self.engine.game.turn {
                    PieceColor::White => (wtime, winc),
                    PieceColor::Black => (btime, binc),
                };

                log!("time: {:?}; inc: {:?}", time, inc);

                let engine = &mut self.engine;
                let best_move = match engine.minimax(self.depth) {
                    Some(m) => m,
                    None => {
                        log!("No engine move found. Maybe the game is finished?");
                        log!("Game state: {:?}", engine.game.state);
                        return (out, UciHandleAction::Continue);
                    }
                };

                let best_move_uci = best_move.to_uci(&engine.game);
                log!("Playing engine move: {}", best_move);
                log!("Fen before playing the move: {}", engine.game.to_fen());
                uci_send!("bestmove {}", best_move_uci);
                engine.game.play(&best_move);
            }
        }

        (out, UciHandleAction::Continue)
    }
}
