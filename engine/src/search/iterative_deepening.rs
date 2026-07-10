use std::{thread, time::Duration};

use crate::{
    engine::Engine,
    move_result::{SearchResult, Terminal},
    platform_timer,
    timers::{MoveTimer, infinite::Infinite, remote::Remote},
};

impl Engine {
    /// Same as `search` but you can use your own timer
    pub fn search_with_timer<T: MoveTimer>(&mut self, timer: &T, max_depth: u8) -> SearchResult {
        let mut depth = 0;
        let mut result = SearchResult::default();

        loop {
            let child = self.minimax(timer, depth);
            result.nodes += child.nodes;

            if child.terminal == Terminal::Timer {
                result.terminal = Terminal::Timer;
                break;
            }

            result.depth = result.depth.max(child.depth);

            if child.best.is_none() {
                break;
            }

            result.best = child.best;
            result.score = child.score;

            if depth == max_depth {
                break;
            }

            depth += 1;
        }

        result
    }

    pub fn threaded_search<T: MoveTimer>(
        &mut self,
        timer: &T,
        max_depth: u8,
        threads: usize,
    ) -> SearchResult {
        let remote = Remote::default();
        let handles = (1..threads)
            .map(|_| {
                let mut engine = self.clone();
                let remote = remote.clone();
                thread::spawn(move || engine.search_with_timer(&remote, max_depth))
            })
            .collect::<Vec<_>>();

        let mut result = self.search_with_timer(timer, max_depth);
        remote.stop();

        for h in handles {
            let Ok(extra) = h.join() else {
                continue;
            };

            result.nodes += extra.nodes;
        }

        result
    }

    /// Searches for the best move in the position until the depth is reached or the duration is up
    pub fn search(&mut self, duration: Duration, max_depth: u8) -> SearchResult {
        if duration == Duration::MAX {
            self.search_with_timer(&Infinite, max_depth)
        } else {
            self.search_with_timer(&platform_timer!(duration), max_depth)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::timers::elapsed::Elapsed;

    use super::*;

    #[track_caller]
    fn assert_iterative_deepening_timing<T: MoveTimer, M: FnOnce(Duration) -> T>(make_timer: M) {
        let mut engine = Engine::default();

        let duration = Duration::from_millis(1000);
        let min = Duration::from_micros((duration.as_micros() as f64 * 0.98) as u64);
        let max = Duration::from_micros((duration.as_micros() as f64 * 1.02) as u64);

        let timer = make_timer(duration);
        let now = Instant::now();
        assert!(!timer.over());
        let _ = engine.search_with_timer(&timer, u8::MAX);
        assert!(timer.over());
        let elapsed = now.elapsed();

        assert!(
            elapsed > min,
            "iterative_deepening for {:?} should have completed after {:?}, but took {:?}",
            duration,
            min,
            elapsed
        );

        assert!(
            elapsed < max,
            "iterative_deepening for {:?} should have completed within {:?}, but took {:?}",
            duration,
            max,
            elapsed
        );
    }

    #[test]
    fn iterative_deepening_should_take_the_right_amount_of_time_on_elapsed() {
        assert_iterative_deepening_timing(Elapsed::now);
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn iterative_deepening_should_take_the_right_amount_of_time_on_rdtsc() {
        assert_iterative_deepening_timing(crate::timers::rdtsc::Rdtsc::now);
    }

    #[test]
    fn iterative_deepening_should_take_the_right_amount_of_time_on_platform() {
        assert_iterative_deepening_timing(|duration| platform_timer!(duration));
    }

    #[test]
    fn iterative_deepening_finds_a_move() {
        let mut engine = Engine::default();
        let duration = Duration::from_millis(200);
        let best_move = engine.search(duration, u8::MAX).best;
        assert!(best_move.is_some());
    }
}
