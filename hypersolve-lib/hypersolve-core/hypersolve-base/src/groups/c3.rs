use std::ops::Mul;

use super::GenericPermutation;

/// Elements of the [cyclic group of order 3](http://escarbille.free.fr/group/?g=3_1)
#[derive(Debug, Default, Copy, Clone, PartialEq, strum::FromRepr, const_gen::CompileConst)]
#[repr(u8)]
pub enum C3 {
    #[default]
    E = 0,
    A = 1,
    AA = 2,
}

impl C3 {
    pub const IDENTITY: Self = C3::E;

    /// # Safety
    /// This is safe as long as `0 <= discriminant <= 2`
    #[inline(always)]
    pub const unsafe fn from_repr_unchecked(discriminant: u8) -> Self {
        std::mem::transmute(discriminant)
    }

    pub const fn group_mul(a: Self, b: Self) -> Self {
        // SAFETY: % 3 ensures that discriminant is between 0 and 2 inclusive
        unsafe { Self::from_repr_unchecked((a as u8 + b as u8) % 3) }
    }

    pub const fn inverse(&self) -> Self {
        const INVERSE_LOOKUP: [C3; 3] = [C3::E, C3::AA, C3::A];

        INVERSE_LOOKUP[*self as u8 as usize]
    }

    pub const fn to_permutation(self) -> GenericPermutation<3> {
        use C3::*;
        // SAFTEY: these are all valid permutations
        unsafe {
            GenericPermutation::from_array_unchecked(match self {
                E => [0, 1, 2],
                A => [1, 2, 0],
                AA => [2, 0, 1],
            })
        }
    }

    /// # Safety
    /// This is safe as long as `value` is a cyclic permutation
    pub const unsafe fn from_permutation_unchecked(value: GenericPermutation<3>) -> Self {
        use C3::*;
        match value.as_array() {
            [0, 1, 2] => E,
            [1, 2, 0] => A,
            [2, 0, 1] => AA,
            _ => std::hint::unreachable_unchecked(),
        }
    }

    pub const fn to_a4(self) -> super::A4 {
        // SAFETY: self <= 2 <= 11
        unsafe { super::A4::from_repr_unchecked(self as u8) }
    }
}

impl Mul for C3 {
    type Output = C3;
    fn mul(self, rhs: Self) -> Self::Output {
        C3::group_mul(self, rhs)
    }
}
