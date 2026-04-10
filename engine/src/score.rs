use std::fmt;
use whalecrab_lib::{implement_operations, movegen::pieces::piece::PieceColor};

#[derive(Debug, Clone, Copy, Default)]
pub struct Score(i32);

implement_operations!(Score, Self, [Eq, Ord, Neg]);
implement_operations!(
    Score,
    Self,
    i32,
    [
        Add, AddAssign, Sub, SubAssign, Mul, PartialEq, PartialOrd, Div
    ]
);

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pawns = self.0 / 100;
        let centipawns = self.0 % 100;
        write!(f, "{}.{}", pawns, centipawns.abs())
    }
}

impl Score {
    pub const MAX: Score = Score(i32::MAX);
    pub const MIN: Score = Score(i32::MIN);

    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub fn for_color(self, color: PieceColor) -> Self {
        match color {
            PieceColor::White => self,
            PieceColor::Black => -self,
        }
    }

    pub const fn to_int(self) -> i32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(Score::new(52019).to_string(), "520.19".to_string());
        assert_eq!(Score::new(-52019).to_string(), "-520.19".to_string());
    }
}
