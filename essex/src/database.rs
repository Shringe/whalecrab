use std::{
    collections::HashMap,
    io,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub seed: u32,
    pub positions: u32,
    pub last_move: String,
    pub last_move_uci: String,
    pub fen: String,
    pub error: String,
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
