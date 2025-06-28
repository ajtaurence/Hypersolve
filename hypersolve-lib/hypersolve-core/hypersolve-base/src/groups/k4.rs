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
    /// This is safe as long as `0 <= discriminant <= 3`
    #[inline(always)]
    pub const unsafe fn from_repr_unchecked(discriminant: u8) -> Self {
        std::mem::transmute(discriminant)
    }

    pub const fn group_mul(a: Self, b: Self) -> Self {
        // SAFETY: XOR operation preserves discriminant range
        unsafe { Self::from_repr_unchecked(a as u8 ^ b as u8) }
    }

    #[inline(always)]
    pub const fn inverse(self) -> Self {
        self
    }

    pub const fn to_a4(self) -> super::A4 {
        // SAFETY: self has max value of 3 so self * 3 has max value of 9 <= 11
        unsafe { super::A4::from_repr_unchecked(self as u8 * 3) }
    }
}

impl Mul for K4 {
    type Output = K4;
    fn mul(self, rhs: Self) -> Self::Output {
        K4::group_mul(self, rhs)
    }
}
