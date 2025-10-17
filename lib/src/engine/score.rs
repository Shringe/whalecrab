use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Score(f32);

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
