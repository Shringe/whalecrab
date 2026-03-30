use std::time::Duration;

use whalecrab_engine::engine::Engine;

/// Stores the state of the uci interface
pub struct UciInterface {
    pub engine: Engine,
    pub depth: u16,
    pub duration: Duration,
}

impl Default for UciInterface {
    fn default() -> Self {
        Self {
            engine: Engine::default(),
            depth: 20,
            duration: Duration::from_mins(1),
        }
    }
}
