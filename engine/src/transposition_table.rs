use whalecrab_lib::movegen::moves::Move;

use crate::{engine::TRANSPOSITION_TABLE_MEMORY_BUDGET_IN_KILOBYTES, score::Score};

#[derive(Default, Clone, Debug, PartialEq)]
pub(crate) struct TranspositionTableEntry {
    pub(crate) best_move: Option<Move>,
    pub(crate) depth: u8,
    pub(crate) score: Score,
    pub(crate) node_type: NodeType,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub(crate) enum NodeType {
    #[default]
    Exact,
    /// A beta cutoff was performed in maxi
    Cut,
    /// An alpha cutoff was performed in mini
    All,
}

type FullEntry = Option<(TranspositionTableEntry, u64)>;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TranspositionTable {
    entries: Box<[FullEntry]>,
    mask: usize,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl TranspositionTable {
    fn from_size(kilobytes: usize) -> Self {
        let entry_size = std::mem::size_of::<FullEntry>();
        let count = (kilobytes * 1024 / entry_size).next_power_of_two();
        Self {
            entries: vec![None; count].into_boxed_slice(),
            mask: count - 1,
        }
    }

    pub(crate) fn new() -> Self {
        let kilobytes = *TRANSPOSITION_TABLE_MEMORY_BUDGET_IN_KILOBYTES.get_or_init(|| {
            (if cfg!(test) && cfg!(debug_assertions) {
                128
            } else if cfg!(test) {
                256
            } else if cfg!(debug_assertions) {
                2048
            } else {
                4096
            }) * 1024
        });

        Self::from_size(kilobytes)
    }

    pub(crate) fn get(&self, hash: u64) -> Option<&TranspositionTableEntry> {
        let key = hash as usize & self.mask;
        let (entry, checksum) = self.entries[key].as_ref()?;
        if *checksum == hash { Some(entry) } else { None }
    }

    pub(crate) fn insert(&mut self, hash: u64, entry: TranspositionTableEntry) {
        let key = hash as usize & self.mask;
        self.entries[key] = Some((entry, hash));
    }

    pub(crate) fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = None;
        }
    }
}
