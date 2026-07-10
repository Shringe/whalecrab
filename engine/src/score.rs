use std::fmt;
use whalecrab_lib::{implement_operations, movegen::pieces::piece::PieceColor};

/// An altenative way to track color which allows negating scores more efficiently than
/// `PieceColor`.
/// This is essentially a `Score`, but with the only valid variants being -1 and 1.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScoreColor(Score);

impl std::ops::Neg for ScoreColor {
    type Output = ScoreColor;
    fn neg(self) -> ScoreColor {
        ScoreColor(-self.0)
    }
}

impl std::ops::Mul<Score> for ScoreColor {
    type Output = Score;
    fn mul(self, rhs: Score) -> Score {
        self.0 * rhs
    }
}

impl std::ops::Mul<ScoreColor> for Score {
    type Output = Score;
    fn mul(self, rhs: ScoreColor) -> Score {
        self * rhs.0
    }
}

impl std::ops::MulAssign<ScoreColor> for Score {
    fn mul_assign(&mut self, rhs: ScoreColor) {
        *self *= rhs.0;
    }
}

impl ScoreColor {
    pub const WHITE: ScoreColor = ScoreColor(Score::new(1));
    pub const BLACK: ScoreColor = ScoreColor(Score::new(-1));

    pub const fn from_color(color: PieceColor) -> ScoreColor {
        match color {
            PieceColor::White => ScoreColor::WHITE,
            PieceColor::Black => ScoreColor::BLACK,
        }
    }

    /// This method will panic if `self` is not `ScoreColor::WHITE` or `ScoreColor::BLACK`
    pub const fn to_color(self) -> PieceColor {
        match self {
            ScoreColor::WHITE => PieceColor::White,
            ScoreColor::BLACK => PieceColor::Black,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Score(i16);

implement_operations!(Score, Self, [Ord, Neg]);
implement_operations!(Score, i16, [PartialEq]);
implement_operations!(
    Score,
    Self,
    i16,
    [
        Add, AddAssign, Sub, SubAssign, Mul, MulAssign, PartialOrd, Div
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
    pub const MAX: Score = Score::new(30_000);
    pub const MIN: Score = Score::new(-Score::MAX.to_int());

    pub const fn new(value: i16) -> Self {
        Self(value)
    }

    pub fn for_color(self, color: PieceColor) -> Self {
        match color {
            PieceColor::White => self,
            PieceColor::Black => -self,
        }
    }

    pub const fn to_int(self) -> i16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(Score::new(5019).to_string(), "50.19".to_string());
        assert_eq!(Score::new(-5019).to_string(), "-50.19".to_string());
    }
}
