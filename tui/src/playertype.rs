use std::time::Duration;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PlayerType {
    Human,
    Engine { search_time: Duration },
}

impl PlayerType {
    pub fn cycle(&mut self) {
        *self = match self {
            PlayerType::Human => PlayerType::Engine {
                search_time: Duration::from_secs(3),
            },
            PlayerType::Engine { .. } => PlayerType::Human,
        };
    }
}
