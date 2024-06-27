use std::{
    fmt::Display,
    ops::{Mul, Neg},
};

use const_for::const_for;

/// An enum representing a positive or negative sign
#[derive(Debug, Default, Copy, Clone, PartialEq, strum::EnumIs)]
#[repr(i8)]
pub enum Sign {
    #[default]
    Pos = 1,
    Neg = -1,
}

impl Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Sign::*;
        let string = match self {
            Pos => "+",
            Neg => "-",
        };
        write!(f, "{}", string)
    }
}

impl Neg for Sign {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Sign::Pos => Sign::Neg,
            Sign::Neg => Sign::Pos,
        }
    }
}

impl Mul for Sign {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match rhs {
            Sign::Pos => self,
            Sign::Neg => -self,
        }
    }
}

impl Sign {
    pub const fn mul(self, other: Self) -> Self {
        match other {
            Sign::Pos => self,
            Sign::Neg => self.other(),
        }
    }

    pub const fn product<const N: usize>(array: [Self; N]) -> Self {
        let mut prod = Self::Pos;

        const_for!(i in 0..N => {
            prod = prod.mul(array[i]);
        });

        prod
    }

    /// Returns the other sign
    pub const fn other(self) -> Self {
        match self {
            Sign::Pos => Sign::Neg,
            Sign::Neg => Sign::Pos,
        }
    }

    /// Converts the sign to a binary value (Pos => 0, Neg => 1)
    pub const fn to_binary(self) -> usize {
        match self {
            Self::Pos => 0,
            Self::Neg => 1,
        }
    }

    /// Converts the binary value to a sign (0 => Pos, 1 => Neg)
    ///
    /// # Safety
    /// Passing a value that is not 0 or 1 results in undefined behavior
    pub const unsafe fn from_binary_unchecked(value: usize) -> Self {
        debug_assert!(matches!(value, 0 | 1));

        match value {
            0 => Self::Pos,
            1 => Self::Neg,
            _ => std::hint::unreachable_unchecked(),
        }
    }
}

/// An enum representing a positive sign, negative sign, or zero
#[derive(Debug, Default, Copy, Clone)]
#[repr(i8)]
pub enum ZeroOrSign {
    #[default]
    Zero = 0,
    Pos = 1,
    Neg = -1,
}

impl Display for ZeroOrSign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ZeroOrSign::*;
        let string = match self {
            Zero => "0",
            Pos => "+",
            Neg => "-",
        };
        write!(f, "{}", string)
    }
}

impl Neg for ZeroOrSign {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            ZeroOrSign::Zero => ZeroOrSign::Zero,
            ZeroOrSign::Pos => ZeroOrSign::Neg,
            ZeroOrSign::Neg => ZeroOrSign::Pos,
        }
    }
}

impl Mul for ZeroOrSign {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match rhs {
            ZeroOrSign::Zero => ZeroOrSign::Zero,
            ZeroOrSign::Pos => self,
            ZeroOrSign::Neg => -self,
        }
    }
}
