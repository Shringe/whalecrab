use std::{
    mem,
    option::Option,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

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
    // TODO: Are All nodes really unnecessary in negamax?
    #[allow(unused)]
    All,
}

#[derive(Debug)]
struct AtomicEntry {
    entry: AtomicU64,
    checksum: AtomicU64,
}

impl Clone for AtomicEntry {
    fn clone(&self) -> Self {
        Self {
            entry: AtomicU64::new(self.entry.load(Self::ORDERING)),
            checksum: AtomicU64::new(self.checksum.load(Self::ORDERING)),
        }
    }
}

impl PartialEq for AtomicEntry {
    fn eq(&self, other: &Self) -> bool {
        self.entry.load(Self::ORDERING) == other.entry.load(Self::ORDERING)
            && self.checksum.load(Self::ORDERING) == other.checksum.load(Self::ORDERING)
    }
}

impl Default for AtomicEntry {
    fn default() -> Self {
        Self::new(None)
    }
}

impl AtomicEntry {
    const ORDERING: Ordering = Ordering::Relaxed;

    fn new(entry: Option<(TranspositionTableEntry, u64)>) -> Self {
        let (entry, checksum) = match entry {
            Some((e, c)) => (Some(e), c),
            None => (None, 0),
        };
        let entry_bits = unsafe { mem::transmute::<Option<TranspositionTableEntry>, u64>(entry) };
        Self {
            entry: AtomicU64::new(entry_bits),
            checksum: AtomicU64::new(checksum ^ entry_bits),
        }
    }

    fn read(&self) -> Option<(TranspositionTableEntry, u64)> {
        let entry_bits = self.entry.load(Self::ORDERING);
        let entry: Option<TranspositionTableEntry> = unsafe { mem::transmute(entry_bits) };
        entry.map(|e| (e, self.checksum.load(Self::ORDERING) ^ entry_bits))
    }

    fn write(&self, entry: Option<(TranspositionTableEntry, u64)>) {
        let (entry, checksum) = match entry {
            Some((e, c)) => (Some(e), c),
            None => (None, 0),
        };
        let entry_bits = unsafe { mem::transmute::<Option<TranspositionTableEntry>, u64>(entry) };
        self.entry.store(entry_bits, Self::ORDERING);
        self.checksum.store(checksum ^ entry_bits, Self::ORDERING);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TranspositionTable {
    entries: Arc<[AtomicEntry]>,
    mask: usize,
    #[cfg(debug_assertions)]
    pub(crate) prevented_collisions: std::cell::RefCell<usize>,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl TranspositionTable {
    fn from_size(kilobytes: usize) -> Self {
        let entry_size = mem::size_of::<AtomicEntry>();
        let count = (kilobytes * 1024 / entry_size).next_power_of_two();
        Self {
            entries: (0..count).map(|_| AtomicEntry::new(None)).collect(),
            mask: count - 1,
            #[cfg(debug_assertions)]
            prevented_collisions: std::cell::RefCell::new(0),
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

    pub(crate) fn get(&self, hash: u64) -> Option<TranspositionTableEntry> {
        let key = hash as usize & self.mask;
        let (entry, checksum) = self.entries[key].read()?;
        if checksum == hash {
            Some(entry)
        } else {
            #[cfg(debug_assertions)]
            {
                *self.prevented_collisions.try_borrow_mut().ok()? += 1;
            }
            None
        }
    }

    pub(crate) fn insert(&self, hash: u64, entry: TranspositionTableEntry) {
        let key = hash as usize & self.mask;
        self.entries[key].write(Some((entry, hash)));
    }

    pub(crate) fn clear(&self) {
        for slot in self.entries.iter() {
            slot.write(None);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::atomic::AtomicU64, time::Duration};

    use crate::engine::Engine;

    use super::*;

    fn play_game(engine: &mut Engine, depth_per_move: u8) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        while let Some(m) = engine.search(Duration::MAX, depth_per_move).best {
            engine.game.play(&m);
            moves.push(m);
        }
        moves
    }

    #[test]
    fn memory_layout() {
        let full_entry_size = size_of::<Option<(TranspositionTableEntry, u64)>>();
        let maybe_entry_size = size_of::<Option<TranspositionTableEntry>>();
        let entry_size = size_of::<TranspositionTableEntry>();
        // created from (Option<T>, u64)
        let full_atomic_entry_size = size_of::<(AtomicU64, AtomicU64)>();
        let full_atomic_entry_size_real = size_of::<AtomicEntry>();
        assert_eq!(
            maybe_entry_size, entry_size,
            "T does not support null pointer optimization"
        );
        assert_eq!(full_entry_size, 16);
        assert_eq!(full_atomic_entry_size, full_entry_size);
        assert_eq!(full_atomic_entry_size, full_atomic_entry_size_real);
    }

    #[test]
    fn pack_and_unpack_default_entry() {
        let entry = Some((TranspositionTableEntry::default(), 0));
        assert_eq!(entry.clone(), AtomicEntry::new(entry).read());
    }

    #[ignore = "View output"]
    #[test]
    fn canary_count_hash_collisions() {
        let mut engine = Engine::default();
        assert_eq!(*engine.transposition_table.prevented_collisions.borrow(), 0);
        play_game(&mut engine, 3);
        panic!(
            "Number of prevented collisions: {}",
            *engine.transposition_table.prevented_collisions.borrow()
        );
    }
}
