use std::any::type_name;
use std::fmt::Display;

use crate::board::Board;
use crate::game::Game;
use crate::movegen::moves::Move;

/// Formats the items in the vector neatly with their native display methods
#[track_caller]
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

/// Adds an error message to the vector if the field of the two structs are found to be different
macro_rules! assert_push {
    ($vec: expr, $expected: expr, $actual: expr, $field: ident, $fmt: tt) => {{
        if $expected.$field != $actual.$field {
            $vec.push(format!(
                concat!(
                    "{} has changed:\nExpected:\n",
                    $fmt,
                    "\nFound:\n",
                    $fmt,
                    "\n"
                ),
                stringify!($field),
                $expected.$field,
                $actual.$field
            ));
        }
    }};
    ($vec: expr, $expected: expr, $actual: expr, $field: ident) => {{
        assert_push!($vec, $expected, $actual, $field, "{}");
    }};
}

/// Compares two games thoroughly and panics if any differences are found
#[track_caller]
pub fn compare_games(before: &Game, after: &Game) {
    let mut differences = Vec::new();

    assert_push!(differences, before, after, position, "{:?}");

    assert_push!(differences, before, after, white_occupied);
    assert_push!(differences, before, after, black_occupied);
    assert_push!(differences, before, after, occupied);

    assert_push!(differences, before, after, white_attacks);
    assert_push!(differences, before, after, black_attacks);
    assert_push!(differences, before, after, white_check_rays);
    assert_push!(differences, before, after, black_check_rays);

    assert_push!(
        differences,
        before.position,
        after.position,
        seen_positions,
        "{:?}"
    );
    assert_push!(differences, before, after, position_history, "{:?}");
    assert_push!(differences, before, after, transposition_table, "{:?}");

    if !differences.is_empty() {
        panic!(
            "Games differ in {} field(s):\n{}",
            differences.len(),
            differences.join("\n")
        );
    }
}

/// Compares and actual board to one generated from a fen
#[track_caller]
pub fn compare_to_fen(board: &Board, fen: &str) {
    let fen_board = &Board::from_fen(fen).unwrap();
    assert_eq!(board, fen_board);
}

/// Asserts that moves contains m
#[track_caller]
pub fn should_generate(moves: &Vec<Move>, m: &Move) {
    assert!(
        moves.contains(m),
        "The valid move {} was not generated! Available {}",
        m,
        format_pretty_list(moves)
    );
}

/// Asserts that moves doesn't contain m
#[track_caller]
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
            "whalecrab_lib::square::Square list [
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
            "whalecrab_lib::square::Square list [];"
        );
    }

    #[test]
    fn compare_to_fen() {
        super::compare_to_fen(&Board::default(), STARTING_FEN);
    }
}
