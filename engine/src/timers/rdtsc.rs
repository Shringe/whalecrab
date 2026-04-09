#[cfg(target_arch = "x86")]
use std::arch::x86::_rdtsc;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_rdtsc;

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::timers::MoveTimer;

/// The number of clock cycles per each nanosecond
static CYCLES_PER_NANOSECOND: OnceLock<u64> = OnceLock::new();
/// The amount of time to spend estimating CYCLES_PER_NANOSECOND
const CALIBRATION_TIME: Duration = Duration::from_millis(1);
const CALIBRATION_PRECISION: u64 = Duration::from_secs(1).as_nanos() as u64;

/// Calculates the TSC frequency
fn calibrate_tsc_frequency(calibration_time: Duration) -> u64 {
    let tsc_start = unsafe { _rdtsc() };
    let wall_start = Instant::now();

    std::thread::sleep(calibration_time);

    let tsc_elapsed = unsafe { _rdtsc() }.wrapping_sub(tsc_start);
    let ns_elapsed = wall_start.elapsed().as_nanos() as u64;

    // cycles/sec = cycles / (ns / 1e9) = cycles * 1e9 / ns
    tsc_elapsed * CALIBRATION_PRECISION / ns_elapsed
}

/// Calibrates the TSC frequency in Nanohertz by comparing it against
/// `Instant` over a short sleep. Called once and cached.
fn tsc_frequency() -> u64 {
    *CYCLES_PER_NANOSECOND.get_or_init(|| calibrate_tsc_frequency(CALIBRATION_TIME))
}

/// An rdtsc-based move timer. Unsafe but extremely low overhead on calling `Rdtsc.over()`.
/// Requires an invariant TSC (virtually all CPUs since ~2008).
/// WARNING: Creating this type for the first time induces a small calibration period to sync this type
/// up with the TSC frequency of the system. The frequency will then be cached and future `Rdtsc`
/// timers will be cheap to create.
pub struct Rdtsc {
    deadline: u64,
}

impl Rdtsc {
    pub fn now(duration: Duration) -> Rdtsc {
        let freq = tsc_frequency();
        let duration_cycles = (duration.as_nanos() * freq as u128) / CALIBRATION_PRECISION as u128;
        let deadline = unsafe { _rdtsc() }.wrapping_add(duration_cycles as u64);
        Rdtsc { deadline }
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

    const TSC_VARIANCE_TOLERANCE: f64 = 0.001;

    fn timeit<T, F: FnOnce() -> T>(f: F) -> (Duration, T) {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        (elapsed, result)
    }

    #[track_caller]
    fn test_compare_calibrations_at_different_durations(left: Duration, right: Duration) {
        let left_freq = calibrate_tsc_frequency(left) as f64;
        let right_freq = calibrate_tsc_frequency(right) as f64;
        let ratio = (left_freq - right_freq).abs() / right_freq;
        assert!(
            ratio < TSC_VARIANCE_TOLERANCE,
            "calibration at {:?} ({}) and {:?} ({}) differ by {:.5}%, exceeding tolerance of {:.5}%",
            left,
            left_freq,
            right,
            right_freq,
            ratio * 100.0,
            TSC_VARIANCE_TOLERANCE * 100.0,
        );
    }

    #[test]
    fn rdtsc_agrees_with_elapsed() {
        let duration = Duration::from_millis(10);

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

    #[test]
    fn cached_tsc_freq_equals_manual_calibration() {
        let uncached = calibrate_tsc_frequency(CALIBRATION_TIME) as f64;
        let cached = tsc_frequency() as f64;
        let ratio = (cached - uncached).abs() / uncached;
        assert!(
            ratio < TSC_VARIANCE_TOLERANCE,
            "cached frequency ({}) and manual calibration ({}) differ by {:.5}%, exceeding tolerance of {:.5}%",
            cached,
            uncached,
            ratio * 100.0,
            TSC_VARIANCE_TOLERANCE * 100.0,
        );
    }

    #[test]
    fn cached_tsc_freq_is_cached() {
        let _ = tsc_frequency();
        let max = Duration::from_micros(1);
        for _ in 0..10 {
            let elapsed = timeit(tsc_frequency).0;
            assert!(elapsed < max);
        }
    }

    #[test]
    #[ignore]
    fn platform_minimum_sleep_duration() {
        let duration = Duration::from_nanos(1);
        let max = Duration::from_nanos(2);

        let elapsed = timeit(|| std::thread::sleep(duration)).0;

        assert!(
            elapsed <= max,
            "Meant to sleep {:?}, slept {:?}, which is more than the maximum acceptable value of {:?}\nThe platform minimum sleep duration is likely {:?}",
            duration,
            elapsed,
            max,
            elapsed,
        );
    }

    #[test]
    fn current_calibration_time_is_sufficient() {
        test_compare_calibrations_at_different_durations(
            CALIBRATION_TIME,
            Duration::from_millis(50),
        );
    }

    #[test]
    #[ignore]
    fn current_calibration_time_is_sufficient_extended() {
        test_compare_calibrations_at_different_durations(
            CALIBRATION_TIME,
            Duration::from_millis(5000),
        );
    }
}
