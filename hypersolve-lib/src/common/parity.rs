/// Even or odd parity
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Parity {
    Even,
    Odd,
}

impl std::ops::Mul for Parity {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            Self::Even => rhs,
            Self::Odd => rhs.opposite(),
        }
    }
}

impl Parity {
    /// Returns whether the parity is odd
    pub(crate) fn is_odd(&self) -> bool {
        matches!(self, Self::Odd)
    }

    /// Returns the opposite parity
    pub(crate) fn opposite(&self) -> Self {
        match self {
            Self::Even => Self::Odd,
            Self::Odd => Self::Even,
        }
    }
}
