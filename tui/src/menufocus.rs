#[derive(Debug, PartialEq)]
pub enum MenuFocus {
    Start,
    Resume,
    Quit,
    White,
    Black,
}

impl MenuFocus {
    pub fn cycle(&mut self) {
        *self = match self {
            MenuFocus::Start => MenuFocus::Resume,
            MenuFocus::Resume => MenuFocus::Quit,
            MenuFocus::Quit => MenuFocus::White,
            MenuFocus::White => MenuFocus::Black,
            MenuFocus::Black => MenuFocus::Start,
        };
    }

    pub fn cycle_back(&mut self) {
        *self = match self {
            MenuFocus::Start => MenuFocus::Black,
            MenuFocus::Resume => MenuFocus::Start,
            MenuFocus::Quit => MenuFocus::Resume,
            MenuFocus::White => MenuFocus::Quit,
            MenuFocus::Black => MenuFocus::White,
        };
    }
}
