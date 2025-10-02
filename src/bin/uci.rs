use std::io;
use std::io::BufRead;

const ID_NAME: &str = "whalecrab";
const ID_AUTHOR: &str = "Shringe";

macro_rules! uci_send {
    ($($arg:tt)*) => {{
        eprintln!("Sent: {}", format!($($arg)*));
        println!($($arg)*);
    }};
}

fn main() {
    let stdin = io::stdin();

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

        match line.as_str() {
            "uci" => {
                uci_send!("id name {ID_NAME}");
                uci_send!("id author {ID_AUTHOR}");
                uci_send!("uciok");
            }

            "isready" => uci_send!("readyok"),

            "ucinewgame" => {}

            _ => {
                eprintln!("Failed to understand: {}", line);
            }
        }
    }
}
