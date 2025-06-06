use std::any::type_name;
use std::fmt::Display;

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

#[cfg(test)]
mod tests {
    use crate::square::Square;

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
            "crabfish::square::Square list [
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
            "crabfish::square::Square list [];"
        );
    }
}
