use std::{
    mem::transmute,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

use crate::timers::MoveTimer;

/// The possible signals an `AnalogRemote` can carry
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Signal {
    #[default]
    /// A search has ended or has yet to begun
    Waiting = 0,
    /// A search is being undergone
    Searching = 1,
    /// Search threads should terminate completely
    Terminate = 2,
}

/// This is similar to `Remote`, but has more modes than start and stop
#[derive(Debug, Default, Clone)]
pub struct AnalogRemote(Arc<AtomicU8>);

impl PartialEq for AnalogRemote {
    fn eq(&self, other: &Self) -> bool {
        self.0.load(Ordering::Relaxed) == other.0.load(Ordering::Relaxed)
    }
}

impl MoveTimer for AnalogRemote {
    #[inline(always)]
    fn over(&self) -> bool {
        self.read() != Signal::Searching
    }
}

impl AnalogRemote {
    pub fn read(&self) -> Signal {
        unsafe { transmute(self.0.load(Ordering::Relaxed)) }
    }

    pub fn write(&self, signal: Signal) {
        self.0.store(
            unsafe { transmute::<Signal, u8>(signal) },
            Ordering::Relaxed,
        );
    }
}
