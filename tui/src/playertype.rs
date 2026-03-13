#[derive(Debug, PartialEq)]
pub enum PlayerType {
    Human,
    Engine,
}

impl PlayerType {
    pub fn cycle(&mut self) {
        *self = match self {
            PlayerType::Human => PlayerType::Engine,
            PlayerType::Engine => PlayerType::Human,
        };
    }
}
