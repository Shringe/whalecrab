use std::{fmt, str::FromStr};

/// Enum of supported uci commands to recieve
pub enum UciCommand {
    UciNewGame,
    Uci,
    Quit,
    IsReady,
    Position,
    Go,
}

#[derive(Debug)]
pub enum UciError {
    UnrecognizedCommand(String),
}

impl fmt::Display for UciError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnrecognizedCommand(cmd) => write!(f, "Unrecognized UCI command: '{}'", cmd),
        }
    }
}

impl FromStr for UciCommand {
    type Err = UciError;

    /// Parses Self from a line of recieved uci
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
            "position" => Ok(Self::Position),
            "go" => Ok(Self::Go),
            _ => Err(UciError::UnrecognizedCommand(cmd.to_string())),
        }
    }
}
