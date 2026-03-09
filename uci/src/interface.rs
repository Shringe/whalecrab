use whalecrab_engine::engine::Engine;

/// Stores the state of the uci interface
pub struct UciInterface {
    pub engine: Option<Engine>,
    pub depth: u16,
}
