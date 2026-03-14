use crate::menufocus::MenuFocus;

#[derive(Debug)]
pub enum Focus {
    Board,
    Fen,
    Command,
    Menu { focus: MenuFocus },
}

impl PartialEq for Focus {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Focus::Board, Focus::Board)
                | (Focus::Fen, Focus::Fen)
                | (Focus::Command, Focus::Command)
                | (Focus::Menu { .. }, Focus::Menu { .. })
        )
    }
}

impl Focus {
    pub fn get_default_menu() -> Focus {
        Focus::Menu {
            focus: MenuFocus::Start,
        }
    }
}
