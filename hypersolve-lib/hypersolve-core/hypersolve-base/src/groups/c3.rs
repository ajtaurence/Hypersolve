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
        match C3::from_repr(discriminant) {
            Some(val) => val,
            None => std::hint::unreachable_unchecked(),
        }
    }

    pub const fn group_mul(a: Self, b: Self) -> Self {
        use C3::*;
        match (a, b) {
            (E, val) | (val, E) => val,
            (AA, AA) => A,
            (A, A) => AA,
            (AA, A) | (A, AA) => E,
        }
    }

    pub const fn inverse(&self) -> Self {
        use C3::*;
        match self {
            E => E,
            A => AA,
            AA => A,
        }
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
        use super::A4;
        match self {
            C3::E => A4::E,
            C3::A => A4::R1,
            C3::AA => A4::R2,
        }
    }
}

impl Mul for C3 {
    type Output = C3;
    fn mul(self, rhs: Self) -> Self::Output {
        C3::group_mul(self, rhs)
    }
}
