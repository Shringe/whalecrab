use std::arch::x86_64::_rdtsc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::timers::move_timer::MoveTimer;

static TSC_FREQ: OnceLock<u64> = OnceLock::new();

/// Calibrates the TSC frequency in Hz by comparing it against
/// `Instant` over a short sleep. Called once and cached.
fn tsc_frequency() -> u64 {
    *TSC_FREQ.get_or_init(|| {
        let tsc_start = unsafe { _rdtsc() };
        let wall_start = Instant::now();

        std::thread::sleep(Duration::from_millis(100));

        let tsc_elapsed = unsafe { _rdtsc() }.wrapping_sub(tsc_start);
        let ns_elapsed = wall_start.elapsed().as_nanos() as u64;

        // cycles/sec = cycles / (ns / 1e9) = cycles * 1e9 / ns
        tsc_elapsed * 1_000_000_000 / ns_elapsed
    })
}

/// An rdtsc-based move timer. Unsafe but extremely low overhead.
/// Requires an invariant TSC (virtually all CPUs since ~2008).
pub struct Rdtsc {
    deadline: u64,
}

impl Rdtsc {
    pub fn now(duration: Duration) -> Rdtsc {
        let freq = tsc_frequency();
        let duration_cycles = (duration.as_nanos() as u64 * freq) / 1_000_000_000;
        Rdtsc {
            deadline: unsafe { _rdtsc() }.wrapping_add(duration_cycles),
        }
    }
}

impl MoveTimer for Rdtsc {
    #[inline(always)]
    fn over(&self) -> bool {
        (unsafe { _rdtsc() }) > self.deadline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timers::elapsed::Elapsed;

    #[test]
    fn rdtsc_agrees_with_elapsed() {
        let duration = Duration::from_millis(50);

        // Rdtsc should be constructed first because of calibration time
        let rdtsc_timer = Rdtsc::now(duration);
        let elapsed_timer = Elapsed::now(duration);

        assert!(!elapsed_timer.over(), "Elapsed fired too early");
        assert!(!rdtsc_timer.over(), "Rdtsc fired too early");

        std::thread::sleep(duration / 2);

        assert!(!elapsed_timer.over(), "Elapsed fired too early");
        assert!(!rdtsc_timer.over(), "Rdtsc fired too early");

        std::thread::sleep(duration / 2);

        assert!(elapsed_timer.over(), "Elapsed did not fire");
        assert!(rdtsc_timer.over(), "Rdtsc did not fire");
    }
}
