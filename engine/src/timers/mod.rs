pub mod elapsed;
pub mod infinite;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod rdtsc;

/// Returns the high performance `Rdtsc` timer on supported platforms, otherwise returns an `Elapsed` timer
#[macro_export]
macro_rules! platform_timer {
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
