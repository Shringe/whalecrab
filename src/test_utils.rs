use std::any::type_name;
use std::fmt::Display;

use crate::board::Board;
use crate::movegen::moves::Move;

/// Formats the items in the vector neatly with their native display methods
pub fn format_pretty_list<T: Display>(v: &Vec<T>) -> String {
    let mut lines = Vec::new();
    let title = type_name::<T>().to_owned();
    let start = " list [".to_owned();
    let end = "];".to_owned();
    if v.is_empty() {
        return title + &start + &end;
    }

    lines.push(title + &start);
    for i in v {
        lines.push(format!("  {},", i));
    }
    lines.push(end);

    lines.join("\n")
}

/// Compares and actual board to one generated from a fen
pub fn compare_to_fen(board: &Board, fen: &str) {
    let fen_board = &Board::from_fen(fen).unwrap();
    assert_eq!(board, fen_board);
}

/// Asserts that moves contains m
pub fn should_generate(moves: &Vec<Move>, m: &Move) {
    assert!(
        moves.contains(m),
        "The valid move {} was not generated! Available {}",
        m,
        format_pretty_list(moves)
    );
}

/// Asserts that moves doesn't contain m
pub fn shouldnt_generate(moves: &Vec<Move>, m: &Move) {
    assert!(
        !moves.contains(m),
        "The invalid move {} was generated! Available {}",
        m,
        format_pretty_list(moves)
    );
}

#[cfg(test)]
mod tests {
    use crate::{board::STARTING_FEN, square::Square};

    use super::*;

    #[test]
    fn format_pretty_lists() {
        let with_trait = vec![
            Square::A1,
            Square::F5,
            Square::H3,
            Square::G6,
            Square::D8,
            Square::C1,
        ];

        let empty: Vec<Square> = Vec::new();

        assert_eq!(
            format_pretty_list(&with_trait),
            "whalecrab::square::Square list [
  A1,
  F5,
  H3,
  G6,
  D8,
  C1,
];",
        );

        assert_eq!(
            format_pretty_list(&empty),
            "whalecrab::square::Square list [];"
        );
    }

    #[test]
    fn compare_to_fen() {
        super::compare_to_fen(&Board::default(), STARTING_FEN);
    }
}
