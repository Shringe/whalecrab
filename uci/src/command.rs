use std::{fmt, str::FromStr, time::Duration};

use whalecrab_lib::position::game::STARTING_FEN;

/// Enum of supported uci commands to receive.
/// This behavior is implemented using the below documentation as a reference.
/// https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf
#[derive(Debug, PartialEq)]
pub enum UciCommand {
    UciNewGame,
    Uci,
    Quit,
    IsReady,
    /// The position to set up on the internal board. The engine should start with the given fen,
    /// then play all of the uci moves.
    Position {
        fen: String,
        uci_moves: String,
    },
    Go {
        movetime: Option<Duration>,
        wtime: Option<Duration>,
        btime: Option<Duration>,
        #[allow(dead_code)]
        winc: Option<Duration>,
        #[allow(dead_code)]
        binc: Option<Duration>,
    },
    SetOption {
        name: String,
        value: String,
    },
}

#[derive(Debug)]
pub enum UciError {
    UnrecognizedCommand(String),
    #[allow(unused)] // We might bring this back later
    ParseMove(String),
    ParseOptionName(String),
    ParseOptionValue(String),
}

impl fmt::Display for UciError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnrecognizedCommand(cmd) => write!(f, "Unrecognized UCI command: '{}'", cmd),
            Self::ParseMove(cmd) => write!(f, "Failed to parse move string: '{}'", cmd),
            Self::ParseOptionName(cmd) => write!(f, "Failed to name of setoption: '{}'", cmd),
            Self::ParseOptionValue(cmd) => write!(f, "Failed to value of setoption: '{}'", cmd),
        }
    }
}

impl FromStr for UciCommand {
    type Err = UciError;

    /// Parses Self from a line of received uci
    fn from_str(line: &str) -> Result<Self, UciError> {
        let cmd = match line.split_once(' ') {
            Some(split) => split.0,
            None => line,
        };

        match cmd {
            "ucinewgame" => Ok(Self::UciNewGame),
            "uci" => Ok(Self::Uci),
            "quit" => Ok(Self::Quit),
            "isready" => Ok(Self::IsReady),
            "position" => {
                let remaining = line.split_once("position ").unwrap_or(("", line)).1;
                let (position, moves) = remaining.split_once(" moves ").unwrap_or((remaining, ""));

                let fen = if let Some(fen) = position.strip_prefix("fen ") {
                    fen
                } else {
                    STARTING_FEN
                };

                Ok(Self::Position {
                    fen: fen.to_string(),
                    uci_moves: moves.to_string(),
                })
            }
            "go" => {
                let tokens: Vec<&str> = line.split(' ').collect();

                let parse_token = |key: &str| {
                    tokens
                        .windows(2)
                        .find(|w| w[0] == key)
                        .and_then(|w| w[1].parse::<u64>().ok())
                        .map(Duration::from_millis)
                };

                let parse_inc = |key: &str| {
                    let inc = parse_token(key);
                    if inc == Some(Duration::ZERO) {
                        None
                    } else {
                        inc
                    }
                };

                Ok(Self::Go {
                    movetime: parse_token("movetime"),
                    wtime: parse_token("wtime"),
                    btime: parse_token("btime"),
                    winc: parse_inc("winc"),
                    binc: parse_inc("binc"),
                })
            }
            "setoption" => {
                let split: Vec<&str> = line.split(' ').collect();
                let name = match split.get(2) {
                    Some(name) => name,
                    None => return Err(UciError::ParseOptionName(line.to_string())),
                };
                let value = match split.get(4) {
                    Some(value) => value,
                    None => return Err(UciError::ParseOptionValue(line.to_string())),
                };

                Ok(Self::SetOption {
                    name: name.to_string(),
                    value: value.to_string(),
                })
            }
            _ => Err(UciError::UnrecognizedCommand(cmd.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::uci;

    use super::*;
    use std::time::Duration;

    #[test]
    fn uci() {
        assert!(matches!(UciCommand::from_str("uci"), Ok(UciCommand::Uci)));
    }

    #[test]
    fn ucinewgame() {
        assert!(matches!(
            UciCommand::from_str("ucinewgame"),
            Ok(UciCommand::UciNewGame)
        ));
    }

    #[test]
    fn quit() {
        assert!(matches!(UciCommand::from_str("quit"), Ok(UciCommand::Quit)));
    }

    #[test]
    fn isready() {
        assert!(matches!(
            UciCommand::from_str("isready"),
            Ok(UciCommand::IsReady)
        ));
    }

    #[test]
    fn position() {
        let fen = "startpos";
        let moves = "e2e4 e7e5";
        let cmd = uci!("position {fen} moves {moves}");

        match cmd {
            UciCommand::Position { fen, uci_moves } => {
                assert_eq!(moves, uci_moves, "Incorrect moves returned: {}", uci_moves);
                assert_ne!(
                    fen, "startpos",
                    "startpos was not converted to a real fen: {}",
                    fen
                );
                assert_eq!(
                    fen, STARTING_FEN,
                    "Did not convert startpos to correct fen: {}",
                    fen
                );
            }
            _ => panic!("Wrong uci command received {:?}", cmd),
        }
    }

    #[test]
    fn position_no_moves() {
        let moves = "";
        let cmd = uci!("position startpos moves");

        match cmd {
            UciCommand::Position { fen, uci_moves } => {
                assert_eq!(moves, uci_moves, "Incorrect moves returned: {}", uci_moves);
                assert_ne!(
                    fen, "startpos",
                    "startpos was not converted to a real fen: {}",
                    fen
                );
                assert_eq!(
                    fen, STARTING_FEN,
                    "Did not convert startpos to correct fen: {}",
                    fen
                );
            }
            _ => panic!("Wrong uci command received {:?}", cmd),
        }
    }

    #[test]
    fn go_bare() {
        let cmd = UciCommand::from_str("go").unwrap();
        assert!(matches!(
            cmd,
            UciCommand::Go {
                movetime: None,
                wtime: None,
                btime: None,
                winc: None,
                binc: None,
            }
        ));
    }

    #[test]
    fn go_movetime() {
        let cmd = UciCommand::from_str("go movetime 1000").unwrap();
        assert!(matches!(
            cmd,
            UciCommand::Go {
                movetime: Some(d),
                ..
            } if d == Duration::from_millis(1000)
        ));
    }

    #[test]
    fn go_wtime_btime() {
        let cmd = UciCommand::from_str("go wtime 60000 btime 60000 winc 500 binc 500").unwrap();
        assert!(matches!(
            cmd,
            UciCommand::Go {
                movetime: None,
                wtime: Some(w),
                btime: Some(b),
                winc: Some(wi),
                binc: Some(bi),
            } if w == Duration::from_millis(60000)
              && b == Duration::from_millis(60000)
              && wi == Duration::from_millis(500)
              && bi == Duration::from_millis(500)
        ));
    }

    #[test]
    fn setoption_depth() {
        let cmd = UciCommand::from_str("setoption name Depth value 5").unwrap();
        assert!(matches!(
            cmd,
            UciCommand::SetOption { name, value } if name == "Depth" && value == "5"
        ));
    }

    #[test]
    fn setoption_movetime() {
        let cmd = UciCommand::from_str("setoption name MaxMoveTimeMs value 3000").unwrap();
        assert!(matches!(
            cmd,
            UciCommand::SetOption { name, value } if name == "MaxMoveTimeMs" && value == "3000"
        ));
    }

    #[test]
    fn unrecognized_command() {
        assert!(matches!(
            UciCommand::from_str("notacommand"),
            Err(UciError::UnrecognizedCommand(_))
        ));
    }

    #[test]
    fn setoption_missing_value() {
        assert!(matches!(
            UciCommand::from_str("setoption name Depth"),
            Err(UciError::ParseOptionValue(_))
        ));
    }

    #[test]
    fn setoption_missing_name() {
        assert!(matches!(
            UciCommand::from_str("setoption"),
            Err(UciError::ParseOptionName(_))
        ));
    }

    #[test]
    fn zero_increment_is_no_increment() {
        let actual = uci!("go winc 50 binc 0");
        let expected = UciCommand::Go {
            movetime: None,
            wtime: None,
            btime: None,
            winc: Some(Duration::from_millis(50)),
            binc: None,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn position_fen_with_spaces() {
        let fen = "k7/pp6/4n3/8/3K1Q2/8/8/R7 w - - 1 2";
        let expected = fen;
        let UciCommand::Position { fen: actual, .. } = uci!("position fen {fen}") else {
            panic!("Wrong cmd returned")
        };
        assert_eq!(actual, expected);
    }
}
