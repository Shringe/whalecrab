use std::io::{BufRead, BufWriter, Write};
use std::{fs::File, io};

use whalecrab_lib::{game::Game, movegen::moves::Move};

const ID_NAME: &str = "whalecrab";
const ID_AUTHOR: &str = "Shringe";

/// Tries to parse milliseconds from uci parameter
fn parse_time_param(param: Option<&str>, name: &str) -> Result<u64, String> {
    let time = param.ok_or_else(|| format!("Missing value for {}", name))?;
    time.parse()
        .map_err(|e| format!("Couldn't parse {} into milliseconds: {}", name, e))
}

// Example uci game for now:
// < uci
// > id name MyEngine
// > id author Me
// > uciok
// < isready
// > readyok
// < ucinewgame
// < position startpos
// < go wtime 300000 btime 300000 winc 0 binc 0
// > bestmove e2e4
// < position startpos moves e2e4 e7e5
// < go wtime 298000 btime 298000 winc 0 binc 0
// > bestmove g1f3
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

    let stdin = io::stdin();
    let mut game = None;
    for line in stdin.lock().lines() {
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

        let cmd = match line.split_once(' ') {
            Some(split) => split.0,
            None => line.as_str(),
        };

        match cmd {
            "quit" => break,
            "uci" => {
                uci_send!("id name {ID_NAME}");
                uci_send!("id author {ID_AUTHOR}");
                uci_send!("uciok");
            }

            "isready" => uci_send!("readyok"),
            "ucinewgame" => game = Some(Game::default()),

            "position" => {
                // TODO, accept positions other than startpos
                // < position startpos moves e2e4 e7e5
                let mut full_cmd = line.split(' ');
                let _ = full_cmd.next();
                let _ = full_cmd.next();
                let _ = full_cmd.next();

                let game: &mut Game = match &mut game {
                    Some(game) => game,
                    None => {
                        log!("Can't accept moves when game is not uninitialized");
                        continue;
                    }
                };

                let uci_moves: Vec<&str> = full_cmd.collect();
                let uci_played = uci_moves.get(game.position.half_move_clock);
                let move_played = match uci_played {
                    Some(uci) => match Move::from_uci(uci, &game.position) {
                        Ok(m) => m,
                        Err(e) => {
                            log!("Failed to parse the played uci move: {:?}", e);
                            continue;
                        }
                    },
                    None => {
                        log!("Couldn't find the move played by uci");
                        continue;
                    }
                };

                game.generate_all_legal_moves();
                log!("Playing move from uci opponent: {}", &move_played);
                game.play(&move_played);
            }

            "go" => match &mut game {
                Some(game) => {
                    let best_move = match game.get_engine_move_minimax(3) {
                        Some(m) => m,
                        None => {
                            log!("No engine move found. Maybe the game is finished?");
                            continue;
                        }
                    };

                    game.play(&best_move);
                    let best_move_uci = best_move.to_uci();
                    log!("Playing engine move: {}", best_move);
                    uci_send!("bestmove {}", best_move_uci);
                }
                None => {
                    log!("Tried to find best move but game is uninitialized");
                    continue;
                }
            },

            _ => {
                log!("Failed to recognize: {}", line);
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
