use std::{
    fmt,
    time::{Duration, Instant},
};

use crate::timers::MoveTimer;

/// An Instant.elapsed() based move timer. Slow but reliable performance
pub struct Elapsed {
    start: Instant,
    duration: Duration,
}

impl MoveTimer for Elapsed {
    #[inline(always)]
    fn over(&self) -> bool {
        self.start.elapsed() > self.duration
    }
}

impl fmt::Display for Elapsed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (elapsed, remaining) = self.stats();
        write!(f, "{:.1?} elapsed, {:.1?} remaining", elapsed, remaining)
    }
}

impl Elapsed {
    pub fn new(start: Instant, duration: Duration) -> Elapsed {
        Elapsed { start, duration }
    }

    pub fn now(duration: Duration) -> Elapsed {
        Elapsed {
            start: Instant::now(),
            duration,
        }
    }

    pub fn remaining(&self) -> Duration {
        self.duration.saturating_sub(self.elapsed())
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Returns `(self.elapsed(), self.remaining())`. The remaining time is computed from the
    /// elaspsed measurement to ensure that `elasped + remaining == self.duration`.
    pub fn stats(&self) -> (Duration, Duration) {
        let elapsed = self.elapsed();
        let remaining = self.duration.saturating_sub(elapsed);
        (elapsed, remaining)
    }
}
