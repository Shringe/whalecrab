use std::{
    collections::HashMap,
    io,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ErrorInfo {
    /// The game ran out of moves but was still set to State::InProgress
    FinishedInProgress { state: String },
    /// The game panicked while playing a move
    PanicOnMove {
        m: String,
        uci: String,
        error: String,
    },
}

impl ErrorInfo {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_error_type_and_string(self) -> (ErrorType, String) {
        match self {
            ErrorInfo::FinishedInProgress { state } => (ErrorType::FinishedInProgress, state),
            ErrorInfo::PanicOnMove { m, uci, error } => {
                (ErrorType::PanicOnMove, format!("{}||{}||{}", m, uci, error))
            }
        }
    }
}

/// Workaround because serde can't deserialize `ErrorInfo`
#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorType {
    FinishedInProgress,
    PanicOnMove,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub seed: u32,
    pub positions: u32,
    pub fen: String,
    pub error: ErrorType,
    pub context: String,
}

pub struct Dataset {
    data: HashMap<u32, Entry>,
}

impl Dataset {
    /// Loads the dataset from a path
    pub fn load(path: &PathBuf) -> Self {
        let data = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)
            .map(|mut r| {
                r.deserialize()
                    .filter_map(|e: Result<Entry, _>| e.ok())
                    .map(|e| (e.seed, e))
                    .collect()
            })
            .unwrap_or_default();
        Self { data }
    }

    /// Saves the dataset back in-place
    pub fn save(&self, path: &PathBuf) -> io::Result<()> {
        let mut writer = csv::WriterBuilder::new()
            .has_headers(true)
            .from_path(path)?;
        for entry in self.data.values() {
            writer.serialize(entry)?;
        }
        Ok(())
    }
}

impl Deref for Dataset {
    type Target = HashMap<u32, Entry>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Dataset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
