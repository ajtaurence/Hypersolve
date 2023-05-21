/// Even or odd parity
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Parity {
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
    pub fn is_even(&self) -> bool {
        match self {
            Parity::Even => true,
            _ => false,
        }
    }

    pub fn is_odd(&self) -> bool {
        match self {
            Parity::Odd => true,
            _ => false,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::Even => Self::Odd,
            Self::Odd => Self::Even,
        }
    }
}
