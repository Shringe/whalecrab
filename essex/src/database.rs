use std::{collections::HashMap, io, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub seed: u64,
    pub positions: usize,
    pub last_move: String,
    pub fen: String,
}

pub fn load(path: &PathBuf) -> HashMap<u64, Entry> {
    csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .map(|mut r| {
            r.deserialize()
                .filter_map(|e: Result<Entry, _>| e.ok())
                .map(|e| (e.seed, e))
                .collect()
        })
        .unwrap_or_default()
}

pub fn save(path: &PathBuf, entries: &HashMap<u64, Entry>) -> io::Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    for entry in entries.values() {
        writer.serialize(entry)?;
    }
    Ok(())
}
