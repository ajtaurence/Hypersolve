use std::ops::Mul;

/// Elements of the [Klein group](http://escarbille.free.fr/group/?g=4_2a)
#[derive(Debug, Default, Copy, Clone, PartialEq, strum::FromRepr, const_gen::CompileConst)]
#[repr(u8)]
pub enum K4 {
    #[default]
    // W sticker is on W axis
    E = 0,
    // W sticker is on Z axis
    U1 = 1,
    // W sticker is on Y/X axis
    U2 = 2,
    // W sticker is on X/Y axis
    U3 = 3,
}

impl K4 {
    pub const IDENTITY: K4 = K4::E;

    /// # Safety
    /// This is safe as long as `0 <= discriminant <= 4`
    #[inline(always)]
    pub const unsafe fn from_repr_unchecked(discriminant: u8) -> Self {
        match K4::from_repr(discriminant) {
            Some(val) => val,
            None => std::hint::unreachable_unchecked(),
        }
    }

    pub const fn group_mul(a: Self, b: Self) -> Self {
        use K4::*;
        match (a, b) {
            (E, val) => val,
            (val, E) => val,
            (U1, U1) | (U2, U2) | (U3, U3) => E,
            (U1, U2) | (U2, U1) => U3,
            (U1, U3) | (U3, U1) => U2,
            (U2, U3) | (U3, U2) => U1,
        }
    }

    #[inline(always)]
    pub const fn inverse(self) -> Self {
        self
    }

    pub const fn to_a4(self) -> super::A4 {
        use super::A4;
        match self {
            K4::E => A4::E,
            K4::U1 => A4::U1,
            K4::U2 => A4::U2,
            K4::U3 => A4::U3,
        }
    }
}

impl Mul for K4 {
    type Output = K4;
    fn mul(self, rhs: Self) -> Self::Output {
        K4::group_mul(self, rhs)
    }
}
