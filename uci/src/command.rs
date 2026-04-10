use std::{fmt, str::FromStr, time::Duration};

use whalecrab_lib::position::game::STARTING_FEN;

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
        /// The fen to a position. Currently will return the starting fen if the GUI says "startpos" instead of a specific fen
        fen: String,
        /// A string such as "e2e4 e7e5 ..."
        uci_moves: String,
    },
    Go {
        /// The exact amount of time that should be searched
        movetime: Option<Duration>,
        /// White's total time on the clock
        wtime: Option<Duration>,
        /// Black's total time on the clock
        btime: Option<Duration>,
        /// White's increment
        #[allow(dead_code)]
        winc: Option<Duration>,
        /// Black's increment
        #[allow(dead_code)]
        binc: Option<Duration>,
        /// The amount of moves left in the time control
        movestogo: Option<u16>,
        /// The maximum depth to search
        depth: Option<u16>,
    },
    SetOption {
        name: String,
        value: String,
    },
}

/// Gets the `n` words after start. Returns None if `start` is not found, or `n` words are not present
/// after start.
fn parse_parameter_n(s: &str, start: &str, n: usize) -> Option<String> {
    let after_start = s.split_once(start)?.1.trim();
    let words: Vec<&str> = after_start.splitn(n.checked_add(1)?, ' ').take(n).collect();
    if words.len() < n {
        None
    } else {
        Some(words.join(" "))
    }
}

/// See `parse_parameter_n`
fn parse_parameter_first(s: &str, start: &str) -> Option<String> {
    parse_parameter_n(s, start, 1)
}

/// Gets all words between `start` and `end`. If it can not find `start`, returns None.
fn parse_parameter(s: &str, start: &str, end: Option<&str>) -> Option<String> {
    let after_start = s.split_once(start)?.1.trim();
    let out = if let Some(end) = end
        && let Some(before_end) = after_start.split_once(end)
    {
        before_end.0
    } else {
        after_start
    };
    Some(out.trim().to_string())
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
                let starting_position = parse_parameter(line, "position", Some("moves"));
                let fen = if let Some(pos) = &starting_position
                    && let Some(fen) = pos.strip_prefix("fen ")
                {
                    fen
                } else {
                    STARTING_FEN
                }
                .to_string();

                let uci_moves = parse_parameter(line, "moves", None).unwrap_or_default();

                Ok(Self::Position { fen, uci_moves })
            }
            "go" => {
                let parse_duration = |key: &str| {
                    eprintln!("{:?}", parse_parameter_first(line, key));
                    parse_parameter_first(line, key)
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(Duration::from_millis)
                };

                let parse_increment = |key: &str| {
                    let inc = parse_duration(key);
                    if inc == Some(Duration::ZERO) {
                        None
                    } else {
                        inc
                    }
                };

                let parse_u16 = |key: &str| {
                    parse_parameter_first(line, key).and_then(|s| s.parse::<u16>().ok())
                };

                Ok(Self::Go {
                    movetime: parse_duration("movetime"),
                    wtime: parse_duration("wtime"),
                    btime: parse_duration("btime"),
                    winc: parse_increment("winc"),
                    binc: parse_increment("binc"),
                    movestogo: parse_u16("movestogo"),
                    depth: parse_u16("depth"),
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
                movestogo: None,
                depth: None,
            }
        ));
    }

    #[test]
    fn go_movetime() {
        assert_eq!(
            uci!("go movetime 1000"),
            UciCommand::Go {
                movetime: Some(Duration::from_millis(1000)),
                wtime: None,
                btime: None,
                winc: None,
                binc: None,
                movestogo: None,
                depth: None,
            }
        );
    }

    #[test]
    fn go_wtime_btime_winc_binc() {
        let cmd = UciCommand::from_str("go wtime 60000 btime 60000 winc 500 binc 500").unwrap();
        assert!(matches!(
            cmd,
            UciCommand::Go {
                movetime: None,
                wtime: Some(w),
                btime: Some(b),
                winc: Some(wi),
                binc: Some(bi),
                movestogo: None,
                depth: None,
            } if w == Duration::from_millis(60000)
              && b == Duration::from_millis(60000)
              && wi == Duration::from_millis(500)
              && bi == Duration::from_millis(500)
        ));
    }

    #[test]
    fn go_wtime_btime_winc_binc_movestogo() {
        let cmd =
            UciCommand::from_str("go wtime 60000 movestogo 567 btime 60000 winc 500 binc 500")
                .unwrap();
        assert!(matches!(
            cmd,
            UciCommand::Go {
                movetime: None,
                wtime: Some(w),
                btime: Some(b),
                winc: Some(wi),
                binc: Some(bi),
                movestogo: Some(mtg),
                depth: None,
            } if w == Duration::from_millis(60000)
              && b == Duration::from_millis(60000)
              && wi == Duration::from_millis(500)
              && bi == Duration::from_millis(500)
              && mtg == 567
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
            movestogo: None,
            depth: None,
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

    #[test]
    fn parse_parameter_movestogo() {
        let line = "go wtime 500 btime 500 movestogo 50";
        let expected = Some("50".to_string());
        let actual = parse_parameter(line, "movestogo", None);
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_parameter_fen_moves() {
        let fen = "k7/pp6/4n3/8/3K1Q2/8/8/R7 w - - 1 2";
        let line = format!("position fen {fen} moves d4e5");
        let expected = Some(fen.to_string());
        let actual = parse_parameter(&line, "fen", Some("moves"));
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_parameter_fen() {
        let fen = "k7/pp6/4n3/8/3K1Q2/8/8/R7 w - - 1 2";
        let line = format!("position fen {fen}");
        let expected = Some(fen.to_string());
        let actual = parse_parameter(&line, "fen", None);
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_parameter_first_movestogo() {
        let line = "go movestogo 50 movetime 1000";
        let expected = Some("50".to_string());
        let actual = parse_parameter_first(line, "movestogo");
        assert_eq!(actual, expected);
    }
}
