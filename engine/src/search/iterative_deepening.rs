use std::time::Duration;

use crate::{
    engine::Engine,
    move_result::{SearchResult, Terminal},
    platform_timer,
    resources::SearchPacket,
    timers::{MoveTimer, infinite::Infinite},
};

/// Allows calling `Engine::search` with either a `MoveTimer` or a `Duration`
trait IntoTimer {
    fn search(self, engine: &mut Engine, max_depth: u8, offset: usize) -> SearchResult;
}

impl<T: MoveTimer> IntoTimer for &T {
    fn search(self, engine: &mut Engine, max_depth: u8, offset: usize) -> SearchResult {
        engine.search_with_timer(self, max_depth, offset)
    }
}

impl IntoTimer for Duration {
    fn search(self, engine: &mut Engine, max_depth: u8, offset: usize) -> SearchResult {
        if self == Duration::MAX {
            engine.search_with_timer(&Infinite, max_depth, offset)
        } else {
            engine.search_with_timer(&platform_timer!(self), max_depth, offset)
        }
    }
}

impl Engine {
    /// Same as `search` but you can use your own timer
    fn search_with_timer<T: MoveTimer>(
        &mut self,
        timer: &T,
        max_depth: u8,
        offset: usize,
    ) -> SearchResult {
        let mut depth = 0;
        let mut result = SearchResult::default();

        let engine = self.clone();
        if let Some(tm) = &mut self.thread_manager {
            tm.start_searching(SearchPacket { engine, max_depth });
        }

        loop {
            let child = self.negamax_threaded(timer, depth, offset);
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

        if let Some(tm) = &mut self.thread_manager {
            tm.stop_searching();
        }

        result
    }

    #[allow(private_bounds)]
    pub(crate) fn search_with_offset(
        &mut self,
        timer: impl IntoTimer,
        max_depth: u8,
        offset: usize,
    ) -> SearchResult {
        timer.search(self, max_depth, offset)
    }

    /// Searches for the best move in the position until the depth is reached or the duration is up
    #[allow(private_bounds)]
    pub fn search(&mut self, timer: impl IntoTimer, max_depth: u8) -> SearchResult {
        self.search_with_offset(timer, max_depth, 0)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::{platform_timer, timers::elapsed::Elapsed};

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
        let _ = engine.search(&timer, u8::MAX);
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

    #[test]
    fn engine_search_supports_both_duration_and_timer() {
        let mut engine = Engine::default();
        let _ = engine.search(&Infinite, 0);
        let _ = engine.search(Duration::from_secs(3), 0);
    }
}
