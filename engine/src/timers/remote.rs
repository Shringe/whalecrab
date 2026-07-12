use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::timers::MoveTimer;

/// A timer that will not stop on its own and needs to be stopped remotely. This can be used to kill
/// search threads from the main thread, or to stop a search early from the UCI.
#[derive(Debug, Default, Clone)]
pub struct Remote(Arc<AtomicBool>);

impl PartialEq for Remote {
    fn eq(&self, other: &Self) -> bool {
        self.0.load(Ordering::Relaxed) == other.0.load(Ordering::Relaxed)
    }
}

impl MoveTimer for Remote {
    #[inline(always)]
    fn over(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

impl Remote {
    /// Stops the timer
    pub fn stop(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Starts the timer
    pub fn start(&self) {
        self.0.store(false, Ordering::Relaxed);
    }
}
