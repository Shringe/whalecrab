use whalecrab_lib::game::Game;

/// Stores the state of the uci interface
pub struct UciInterface {
    pub game: Option<Game>,
    pub depth: u16,
}
