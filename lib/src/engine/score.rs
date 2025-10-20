use std::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Score(i32);

impl Add for Score {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Score(self.0 + other.0)
    }
}

impl Sub for Score {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Score(self.0 - other.0)
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl SubAssign for Score {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pawns = self.0 / 100;
        let centipawns = self.0 % 100;
        write!(f, "{}.{}", pawns, centipawns.abs())
    }
}

impl Default for Score {
    fn default() -> Self {
        Self(0)
    }
}

impl Score {
    pub const MAX: Score = Score(i32::MAX);
    pub const MIN: Score = Score(i32::MIN);

    pub fn new(value: i32) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::score::Score;

    #[test]
    fn display() {
        assert_eq!(Score::new(52019).to_string(), "520.19".to_string());
        assert_eq!(Score::new(-52019).to_string(), "-520.19".to_string());
    }
}
