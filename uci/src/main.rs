mod command;
mod interface;

use std::io::{BufRead, BufWriter, Write};
use std::str::FromStr;
use std::time::Duration;
use std::{fs::File, io};

use whalecrab_engine::engine::Engine;
use whalecrab_lib::movegen::pieces::piece::PieceColor;
use whalecrab_lib::{game::Game, movegen::moves::Move};

use crate::command::UciCommand;
use crate::interface::UciInterface;

const ID_NAME: &str = "whalecrab";
const ID_AUTHOR: &str = "Shringe";

fn main() {
    let logfile = File::create("/tmp/whalecrab_uci.log");
    let mut writer = match logfile {
        Ok(f) => Some(BufWriter::new(f)),
        Err(e) => {
            eprintln!("Can't log to file: {}", e);
            None
        }
    };

    macro_rules! log {
            ($($arg:tt)*) => {{
                let msg = format!($($arg)*) + "\n";
                eprint!("{}", msg);
                if let Some(writer) = &mut writer {
                    if let Err(e) = writer.write_all(msg.as_bytes()) {
                        eprintln!("Couldn't write to log buffer: {}", e);
                    }
                }
            }};
        }

    macro_rules! uci_send {
            ($($arg:tt)*) => {{
                let msg = format!($($arg)*);
                log!("Sent: {}", msg);
                println!("{}", msg);
            }};
        }

    // TODO, allow setoption for depth
    let mut uci = UciInterface::default();

    let stdin = io::stdin();
    'outer: for line in stdin.lock().lines() {
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

        match cmd {
            UciCommand::UciNewGame => uci.engine.with_new_game(Game::default()),
            UciCommand::Quit => break,
            UciCommand::IsReady => uci_send!("readyok"),

            UciCommand::Uci => {
                uci_send!("id name {ID_NAME}");
                uci_send!("id author {ID_AUTHOR}");
                uci_send!("option name Depth type spin default 20 min 0 max 200",);
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
                        uci.depth = depth
                    }
                    Err(e) => {
                        log!("Failed to parse depth: {}", e);
                    }
                },
                "maxmovetimems" => match value.parse::<u64>() {
                    Ok(0) => {
                        log!("Move time limit disabled");
                        uci.duration = Duration::MAX;
                    }
                    Ok(ms) => {
                        log!("Setting max move time to {}ms", ms);
                        uci.duration = Duration::from_millis(ms);
                    }
                    Err(e) => log!("Failed to parse movetime: {}", e),
                },
                _ => {
                    log!("Unknown option: {}", name);
                }
            },

            UciCommand::Position { uci_moves } => {
                let engine = &mut uci.engine;

                // Reset to starting position
                engine.with_new_game(Game::default());

                // Play all moves in sequence
                log!("{:#?}", uci_moves);
                for uci_move in uci_moves.split(' ') {
                    let move_to_play = match Move::from_uci(uci_move, &engine.game) {
                        Ok(m) => m,
                        Err(e) => {
                            log!("Failed to parse uci move '{}': {:?}", uci_move, e);
                            continue 'outer;
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
                let (time, inc) = match uci.engine.game.turn {
                    PieceColor::White => (wtime, winc),
                    PieceColor::Black => (btime, binc),
                };

                log!("time: {:?}; inc: {:?}", time, inc);

                let engine = &mut uci.engine;
                let best_move = match engine.minimax(uci.depth) {
                    Some(m) => m,
                    None => {
                        log!("No engine move found. Maybe the game is finished?");
                        log!("Game state: {:?}", engine.game.state);
                        continue;
                    }
                };

                let best_move_uci = best_move.to_uci(&engine.game);
                log!("Playing engine move: {}", best_move);
                log!("Fen before playing the move: {}", engine.game.to_fen());
                uci_send!("bestmove {}", best_move_uci);
                engine.game.play(&best_move);
            }
        }
    }

    if let Some(writer) = &mut writer {
        match writer.flush() {
            Ok(_) => eprintln!("Log flushed successfully"),
            Err(e) => eprintln!("Failed to flush log file: {}", e),
        }
    }
}
