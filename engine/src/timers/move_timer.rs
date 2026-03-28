pub trait MoveTimer {
    /// Checks if the timer has ran out of time
    fn over(&self) -> bool;
}
