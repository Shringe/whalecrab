use std::io;
use std::io::BufRead;

use whalecrab::game::Game;
use whalecrab::movegen::moves::Move;

const ID_NAME: &str = "whalecrab";
const ID_AUTHOR: &str = "Shringe";

macro_rules! uci_send {
    ($($arg:tt)*) => {{
        eprintln!("Sent: {}", format!($($arg)*));
        println!($($arg)*);
    }};
}

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
    let stdin = io::stdin();
    let mut game = None;
    let mut last_move_uci = None;

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => {
                eprintln!("Recieved: {}", line);
                line
            }
            Err(e) => {
                eprintln!("Failed to read stdin: {}", e);
                continue;
            }
        };

        let cmd = match line.split_once(' ') {
            Some(split) => split.0,
            None => line.as_str(),
        };

        match cmd {
            "uci" => {
                uci_send!("id name {ID_NAME}");
                uci_send!("id author {ID_AUTHOR}");
                uci_send!("uciok");
            }

            "isready" => uci_send!("readyok"),
            "ucinewgame" => {}

            "position" => {
                // TODO, accept positions other than startpos
                // < position startpos moves e2e4 e7e5
                let mut full_cmd = line.split(' ');
                let _ = full_cmd.next();
                let _ = full_cmd.next();
                let wants_init = full_cmd.next();

                if wants_init == Some("moves") {
                    let game: &mut Game = match &mut game {
                        Some(game) => game,
                        None => {
                            eprintln!("Can't accept moves when game is not uninitialized");
                            continue;
                        }
                    };

                    let mut uci_played = None;
                    let mut found_last_move = false;
                    for uci_move in full_cmd {
                        if found_last_move {
                            uci_played = Some(uci_move);
                            break;
                        } else if Some(uci_move.to_string()) == last_move_uci {
                            found_last_move = true;
                        }
                    }

                    let move_played = match uci_played {
                        Some(uci) => match Move::from_uci(uci, &game.position) {
                            Ok(m) => m,
                            Err(e) => {
                                eprintln!("Failed to parse the played uci move: {:?}", e);
                                continue;
                            }
                        },
                        None => {
                            eprintln!("Couldn't find the move played by uci");
                            continue;
                        }
                    };

                    game.generate_all_legal_moves();
                    game.play(&move_played);
                } else {
                    game = Some(Game::default());
                }
            }

            "go" => {
                let mut full_cmd = line.split(' ');
                let _ = full_cmd.next();
                let _ = full_cmd.next();
                let white_time = full_cmd.next();
                let _ = full_cmd.next();
                let black_time = full_cmd.next();
                let _ = full_cmd.next();
                let white_increment = full_cmd.next();
                let _ = full_cmd.next();
                let black_increment = full_cmd.next();

                let _white_time_ms = match parse_time_param(white_time, "wtime") {
                    Ok(ms) => ms,
                    Err(e) => {
                        eprintln!("{}", e);
                        continue;
                    }
                };

                let _black_time_ms = match parse_time_param(black_time, "btime") {
                    Ok(ms) => ms,
                    Err(e) => {
                        eprintln!("{}", e);
                        continue;
                    }
                };

                let _white_time_ms = match parse_time_param(white_time, "wink") {
                    Ok(ms) => ms,
                    Err(e) => {
                        eprintln!("{}", e);
                        continue;
                    }
                };

                let _black_time_ms = match parse_time_param(black_time, "bink") {
                    Ok(ms) => ms,
                    Err(e) => {
                        eprintln!("{}", e);
                        continue;
                    }
                };

                match &mut game {
                    Some(game) => {
                        let best_move = match game.get_engine_move_minimax(3) {
                            Some(m) => m,
                            None => {
                                eprintln!("No engine move found. Maybe the game is finished?");
                                continue;
                            }
                        };

                        game.play(&best_move);
                        let best_move_uci = best_move.to_uci();
                        last_move_uci = Some(best_move_uci.clone());
                        uci_send!("bestmove {}", best_move_uci);
                    }
                    None => {
                        eprintln!("Tried to find best move but game is uninitialized");
                        continue;
                    }
                }
            }

            _ => {
                eprintln!("Failed to recognize: {}", line);
            }
        }
    }
}
