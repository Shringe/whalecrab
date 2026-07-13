use std::{
    num::NonZero,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    thread::{self, Thread},
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
    /// Amount of memory to allocate for the transposition table in kilobytes
    pub memory: usize,
    /// Number of total threads to use for searching
    pub threads: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Self::new(64 * 1024, available_thread_count().into())
    }
}

impl Budget {
    pub fn new(memory: usize, threads: usize) -> Self {
        Self { memory, threads }
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
    /// The number of spawned worker threads currently alive
    active_workers: Arc<AtomicUsize>,
    /// A thread handle to all current workers
    workers: Vec<Thread>,
}

impl Drop for ThreadManager {
    fn drop(&mut self) {
        self.kill_workers();
    }
}

impl PartialEq for ThreadManager {
    fn eq(&self, other: &Self) -> bool {
        self.remote == other.remote
    }
}

impl Clone for ThreadManager {
    fn clone(&self) -> Self {
        Self {
            remote: self.remote.clone(),
            search: self.search.clone(),
            active_workers: self.active_workers.clone(),
            workers: Vec::new(),
        }
    }
}

impl ThreadManager {
    /// This will spawn 1 less workers than `num_threads`
    pub fn spawn_workers(&mut self, num_threads: usize) {
        self.remote.write(Signal::Waiting);
        let threads = num_threads.saturating_sub(1);
        self.workers.reserve_exact(threads);
        for offset in 0..threads {
            let tm = self.clone();
            let handle = thread::spawn(move || {
                tm.worker(offset);
            });
            self.workers.push(handle.thread().clone());
        }
    }

    /// Invalidates the search packet and its transposition table reference
    pub fn invalidate_packet(&self) {
        *self.search.lock().unwrap() = None;
    }

    /// Returns the amount of active workers
    pub fn active_workers(&self) -> usize {
        self.active_workers.load(Ordering::Relaxed)
    }

    pub fn kill_workers(&mut self) {
        self.remote.write(Signal::Terminate);
        while let Some(w) = self.workers.pop() {
            w.unpark();
        }
    }

    /// Blocks the current thread until all workers are killed
    pub fn block_until_workers_are_killed(&self) {
        while self.active_workers() > 0 {}
    }

    /// Unparks worker threads, encouraging them to check the `remote`
    fn notify_workers(&self) {
        for w in &self.workers {
            w.unpark();
        }
    }

    pub fn start_searching(&self, packet: SearchPacket) {
        *self.search.lock().unwrap() = Some(packet);
        self.remote.write(Signal::Searching);
        self.notify_workers();
    }

    pub fn stop_searching(&self) {
        self.remote.write(Signal::Waiting);
    }

    fn worker(&self, offset: usize) {
        self.active_workers.fetch_add(1, Ordering::Relaxed);

        loop {
            match self.remote.read() {
                Signal::Waiting => {
                    thread::park();
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

            engine.search_with_offset(&self.remote, max_depth, offset);
        }

        self.active_workers.fetch_sub(1, Ordering::Relaxed);
    }
}
