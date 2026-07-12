use std::{
    cell::Cell,
    num::NonZero,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use crate::{
    engine::Engine,
    timers::analog_remote::{AnalogRemote, Signal},
};

/// Cached from `thread::available_parallelism()`
static AVAILABLE_THREAD_COUNT: OnceLock<NonZero<usize>> = OnceLock::new();

/// Returns the number of logical cores on the current machine
fn available_thread_count() -> NonZero<usize> {
    *AVAILABLE_THREAD_COUNT.get_or_init(|| {
        #[cfg(not(test))]
        {
            thread::available_parallelism().unwrap_or(NonZero::new(1).unwrap())
        }
        #[cfg(test)]
        {
            NonZero::new(1).unwrap()
        }
    })
}

/// Defines the available resources the engine can use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    /// Amount of memory to allocate for the transposition table
    pub(crate) memory_budget_kilobytes: usize,
    /// Number of total threads to use for searching
    pub(crate) thread_count: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Self::new(64 * 1024, available_thread_count().into())
    }
}

impl Budget {
    pub fn new(memory_budget_kilobytes: usize, thread_count: usize) -> Self {
        Self {
            memory_budget_kilobytes,
            thread_count,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct SearchPacket {
    pub(crate) engine: Engine,
    pub(crate) max_depth: u8,
}

/// Manages extra search threads
#[derive(Debug, Default)]
pub(crate) struct ThreadManager {
    /// Coordinates threads
    remote: AnalogRemote,
    /// Information the main thread passes to extra threads so that they can start their search
    search: Arc<Mutex<Option<SearchPacket>>>,
    /// This increments on every clone. It acts as a unique thread ID that can be used for move
    /// ordering offsets.
    /// The main thread should be careful to not use this value as its offset, and
    /// to use 0 instead.
    thread_number: Cell<usize>,
    /// The number of spawned worker threads currently alive
    active_workers: Arc<AtomicUsize>,
}

impl Drop for ThreadManager {
    fn drop(&mut self) {
        self.kill_workers();
    }
}

impl PartialEq for ThreadManager {
    fn eq(&self, other: &Self) -> bool {
        self.remote == other.remote
            // && self.search == other.search
            && self.thread_number == other.thread_number
    }
}

impl Clone for ThreadManager {
    fn clone(&self) -> Self {
        self.thread_number.update(|n| n.wrapping_add(1));

        Self {
            remote: self.remote.clone(),
            search: self.search.clone(),
            thread_number: self.thread_number.clone(),
            active_workers: self.active_workers.clone(),
        }
    }
}

impl ThreadManager {
    /// This will spawn 1 less workers than `num_threads`
    pub fn spawn_workers(&mut self, num_threads: usize) {
        self.remote.write(Signal::Waiting);
        for _ in 1..num_threads {
            let tm = self.clone();
            thread::spawn(move || {
                tm.worker();
            });
        }
    }

    /// Returns the amount of active workers
    pub fn active_workers(&self) -> usize {
        self.active_workers.load(Ordering::Relaxed)
    }

    pub fn kill_workers(&self) {
        self.remote.write(Signal::Terminate);
    }

    pub fn start_searching(&self, packet: SearchPacket) {
        *self.search.lock().unwrap() = Some(packet);
        self.remote.write(Signal::Searching);
    }

    pub fn stop_searching(&self) {
        self.remote.write(Signal::Waiting);
    }

    fn worker(&self) {
        self.active_workers.fetch_add(1, Ordering::Relaxed);
        let duration = Duration::from_millis(15 + self.thread_number.get() as u64);

        loop {
            match self.remote.read() {
                Signal::Waiting => {
                    thread::sleep(duration);
                    continue;
                }
                Signal::Terminate => break,
                Signal::Searching => {}
            }

            let Some(SearchPacket {
                mut engine,
                max_depth,
            }) = self.search.lock().map(|p| p.clone()).ok().flatten()
            else {
                continue;
            };

            engine.search_with_offset(&self.remote, max_depth, self.thread_number.get());
        }

        self.active_workers.fetch_sub(1, Ordering::Relaxed);
    }
}
