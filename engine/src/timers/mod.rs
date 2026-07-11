use std::time::Duration;

use crate::{engine::Engine, move_result::SearchResult, timers::infinite::Infinite};

pub mod elapsed;
pub mod infinite;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod rdtsc;
pub mod remote;

/// Returns the high performance `Rdtsc` timer on supported platforms, otherwise returns an `Elapsed` timer
#[macro_export]
macro_rules! platform_timer {
    (Duration::MAX) => {{ $crate::timers::infinite::Infinite }};
    (time::Duration::MAX) => {{ $crate::timers::infinite::Infinite }};
    (std::time::Duration::MAX) => {{ $crate::timers::infinite::Infinite }};
    (core::time::Duration::MAX) => {{ $crate::timers::infinite::Infinite }};
    ($duration:expr) => {{
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            $crate::timers::rdtsc::Rdtsc::now($duration)
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            $crate::timers::elapsed::Elapsed::now($duration)
        }
    }};
}

/// Used for timing move searches
pub trait MoveTimer {
    /// Checks if the timer has ran out of time
    fn over(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_timer_optimizes_known_max_durations_to_infinite() {
        let _: Infinite = platform_timer!(Duration::MAX);
        let _: Infinite = platform_timer!(time::Duration::MAX);
        let _: Infinite = platform_timer!(std::time::Duration::MAX);
        let _: Infinite = platform_timer!(core::time::Duration::MAX);
    }
}
