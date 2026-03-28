use std::time::{Duration, Instant};

use crate::timers::move_timer::MoveTimer;

/// An Instant.elapsed() based move timer. Slow but reliable performance
pub struct Elapsed {
    start: Instant,
    duration: Duration,
}

impl Elapsed {
    #[allow(dead_code)]
    pub fn new(start: Instant, duration: Duration) -> Elapsed {
        Elapsed { start, duration }
    }

    pub fn now(duration: Duration) -> Elapsed {
        Elapsed {
            start: Instant::now(),
            duration,
        }
    }
}

impl MoveTimer for Elapsed {
    #[inline(always)]
    fn over(&self) -> bool {
        self.start.elapsed() > self.duration
    }
}
