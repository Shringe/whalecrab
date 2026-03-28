use crate::timers::MoveTimer;

/// A dummy timer that never finishes. Used for depth-only searches
pub struct Infinite;

impl MoveTimer for Infinite {
    #[inline(always)]
    fn over(&self) -> bool {
        false
    }
}
