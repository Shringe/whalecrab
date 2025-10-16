mod command;
mod interface;

use std::io::{BufRead, BufWriter, Write};
use std::str::FromStr;
use std::{fs::File, io};

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
    let mut uci = UciInterface {
        game: None,
        depth: 3,
    };

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
            UciCommand::UciNewGame => uci.game = Some(Game::default()),
            UciCommand::Quit => break,
            UciCommand::IsReady => uci_send!("readyok"),

            UciCommand::Uci => {
                uci_send!("id name {ID_NAME}");
                uci_send!("id author {ID_AUTHOR}");
                uci_send!("uciok");
            }

            UciCommand::Position { uci_moves } => {
                let game = match &mut uci.game {
                    Some(game) => game,
                    None => {
                        log!("Can't accept moves when game is uninitialized");
                        continue;
                    }
                };

                // Reset to starting position
                *game = Game::default();

                // Play all moves in sequence
                log!("{:#?}", uci_moves);
                for uci_move in uci_moves.split(' ') {
                    let move_to_play = match Move::from_uci(uci_move, &game.position) {
                        Ok(m) => m,
                        Err(e) => {
                            log!("Failed to parse uci move '{}': {:?}", uci_move, e);
                            continue 'outer;
                        }
                    };
                    log!("Playing move: {}", move_to_play);
                    game.play(&move_to_play);
                }
                log!("Final position FEN: {}", game.position.to_fen());
                log!("Game state: {:?}", game.position.state);
            }

            UciCommand::Go => match &mut uci.game {
                Some(game) => {
                    let best_move = match game.get_engine_move_minimax(uci.depth) {
                        Some(m) => m,
                        None => {
                            log!("No engine move found. Maybe the game is finished?");
                            log!("Game state: {:?}", game.position.state);
                            continue;
                        }
                    };

                    let best_move_uci = best_move.to_uci();
                    log!("Playing engine move: {}", best_move);
                    log!("Fen before playing the move: {}", game.position.to_fen());
                    uci_send!("bestmove {}", best_move_uci);
                    game.play(&best_move);
                }
                None => {
                    log!("Tried to find best move but game is uninitialized");
                    continue;
                }
            },
        }
    }

    if let Some(writer) = &mut writer {
        match writer.flush() {
            Ok(_) => eprintln!("Log flushed successfully"),
            Err(e) => eprintln!("Failed to flush log file: {}", e),
        }
    }
}
