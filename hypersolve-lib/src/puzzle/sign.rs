/// An enum representing a positive or negative sign
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, strum::EnumIter, strum::EnumCount, Hash)]
#[repr(i8)]
pub enum Sign {
    #[default]
    Pos = 1,
    Neg = -1,
}

impl std::fmt::Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Sign::*;
        let string = match self {
            Pos => "+",
            Neg => "-",
        };
        write!(f, "{}", string)
    }
}

macro_rules! impl_sign_to_from_int {
    ($type:ty) => {
        impl TryFrom<$type> for Sign {
            type Error = String;
            fn try_from(value: $type) -> Result<Self, Self::Error> {
                match value {
                    1 => Ok(Sign::Pos),
                    -1 => Ok(Sign::Neg),
                    _ => Err(format!("cannot convert {:?} to a sign", value)),
                }
            }
        }
        impl From<Sign> for $type {
            fn from(value: Sign) -> Self {
                match value {
                    Sign::Pos => 1,
                    Sign::Neg => -1,
                }
            }
        }
    };
}
for_each!(impl_sign_to_from_int!(i8, i16, i32, i64, i128, isize));

impl std::ops::Neg for Sign {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Sign::Pos => Sign::Neg,
            Sign::Neg => Sign::Pos,
        }
    }
}

impl std::ops::Mul for Sign {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match rhs {
            Sign::Pos => self,
            Sign::Neg => -self,
        }
    }
}

impl Sign {
    /// Converts the sign to a binary value (Pos => 0, Neg => 1)
    pub(crate) const fn to_binary(self) -> usize {
        match self {
            Self::Pos => 0,
            Self::Neg => 1,
        }
    }

    /// Converts the binary value to a sign (0 => Pos, 1 => Neg)
    pub(crate) const fn from_binary(value: usize) -> Self {
        match value {
            0 => Self::Pos,
            1 => Self::Neg,
            _ => panic!("Value is not a binary value"),
        }
    }

    /// Returns whether the sign is positive
    pub const fn is_positive(&self) -> bool {
        matches!(self, Self::Pos)
    }

    /// Returns whether the sign is negative
    pub const fn is_negative(&self) -> bool {
        matches!(self, Self::Neg)
    }
}

/// An enum representing a positive sign, negative sign, or zero
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, strum::EnumIter, strum::EnumCount, Hash)]
#[repr(i8)]
pub(crate) enum ZeroOrSign {
    #[default]
    Zero = 0,
    Pos = 1,
    Neg = -1,
}

impl std::fmt::Display for ZeroOrSign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ZeroOrSign::*;
        let string = match self {
            Zero => "0",
            Pos => "1",
            Neg => "-1",
        };
        write!(f, "{}", string)
    }
}

macro_rules! impl_sign_to_from_int {
    ($type:ty) => {
        impl TryFrom<$type> for ZeroOrSign {
            type Error = String;
            fn try_from(value: $type) -> Result<Self, Self::Error> {
                match value {
                    0 => Ok(ZeroOrSign::Zero),
                    1 => Ok(ZeroOrSign::Pos),
                    -1 => Ok(ZeroOrSign::Neg),
                    _ => Err(format!("cannot convert {:?} to zero or sign", value)),
                }
            }
        }
        impl From<ZeroOrSign> for $type {
            fn from(value: ZeroOrSign) -> Self {
                match value {
                    ZeroOrSign::Zero => 0,
                    ZeroOrSign::Pos => 1,
                    ZeroOrSign::Neg => -1,
                }
            }
        }
    };
}
for_each!(impl_sign_to_from_int!(i8, i16, i32, i64, i128, isize));

impl std::ops::Neg for ZeroOrSign {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            ZeroOrSign::Zero => ZeroOrSign::Zero,
            ZeroOrSign::Pos => ZeroOrSign::Neg,
            ZeroOrSign::Neg => ZeroOrSign::Pos,
        }
    }
}

impl std::ops::Mul for ZeroOrSign {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match rhs {
            ZeroOrSign::Zero => ZeroOrSign::Zero,
            ZeroOrSign::Pos => self,
            ZeroOrSign::Neg => -self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_to_from_binary() {
        use strum::IntoEnumIterator;
        let to_from_binary = |val: Sign| Sign::from_binary(val.to_binary());

        for sign in Sign::iter() {
            assert_eq!(to_from_binary(sign), sign);
        }
    }

    #[test]
    fn sign_to_from_int() {
        assert_eq!(Sign::try_from(1).unwrap(), Sign::Pos);
        assert_eq!(Sign::try_from(-1).unwrap(), Sign::Neg);
        assert_eq!(i32::from(Sign::Pos), 1);
        assert_eq!(i32::from(Sign::Neg), -1);
    }

    #[test]
    fn zero_or_sign_to_from_int() {
        assert_eq!(ZeroOrSign::try_from(0).unwrap(), ZeroOrSign::Zero);
        assert_eq!(ZeroOrSign::try_from(1).unwrap(), ZeroOrSign::Pos);
        assert_eq!(ZeroOrSign::try_from(-1).unwrap(), ZeroOrSign::Neg);
        assert_eq!(i32::from(ZeroOrSign::Zero), 0);
        assert_eq!(i32::from(ZeroOrSign::Pos), 1);
        assert_eq!(i32::from(ZeroOrSign::Neg), -1);
    }
}
