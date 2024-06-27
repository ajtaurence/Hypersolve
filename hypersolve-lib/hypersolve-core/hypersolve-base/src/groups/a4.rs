use std::ops::Mul;

use const_for::const_for;
use strum::VariantArray;

use super::*;

/// Elements of the [A4 group](http://escarbille.free.fr/group/?g=12_3a)
#[derive(Debug, Default, Copy, Clone, PartialEq, strum::VariantArray, const_gen::CompileConst)]
#[repr(u8)]
pub enum A4 {
    #[default]
    E = 0,
    R1 = 1,
    R2 = 2,
    U1 = 3,
    R8 = 4,
    R6 = 5,
    U2 = 6,
    R5 = 7,
    R3 = 8,
    U3 = 9,
    R4 = 10,
    R7 = 11,
}

impl A4 {
    pub const IDENTITY: Self = A4::E;

    pub const fn group_mul(a: A4, b: A4) -> A4 {
        const A4_MUL_TABLE: [[A4; 12]; 12] = {
            let mut result = [[A4::E; 12]; 12];

            const_for!(i in 0..12 => {
                const_for!(j in 0..12 => {
                    let elem1 = A4::VARIANTS[i];
                    let elem2 = A4::VARIANTS[j];

                    let result_elem = unsafe {
                        A4::from_permutation_unchecked(GenericPermutation::group_mul(
                            &elem1.to_permutation(),
                            &elem2.to_permutation(),
                        ))
                    };

                    result[elem1 as usize][elem2 as usize] = result_elem;
                });
            });

            result
        };

        A4_MUL_TABLE[a as usize][b as usize]
    }

    pub const fn from_k4_c3(k4: K4, c3: C3) -> Self {
        // Make sure this is consistent with C3::from(A4)
        match (k4, c3) {
            (K4::E, c3_val) => c3_val.to_a4(),
            (k4_val, C3::E) => k4_val.to_a4(),
            (K4::U1, C3::A) => Self::R8,
            (K4::U1, C3::AA) => Self::R6,
            (K4::U2, C3::A) => Self::R5,
            (K4::U2, C3::AA) => Self::R3,
            (K4::U3, C3::A) => Self::R4,
            (K4::U3, C3::AA) => Self::R7,
        }
    }

    pub const fn inverse(self) -> Self {
        use A4::*;
        match self {
            E => E,
            R1 => R2,
            R2 => R1,
            U1 => U1,
            R8 => R3,
            R6 => R4,
            U2 => U2,
            R5 => R7,
            R3 => R8,
            U3 => U3,
            R4 => R6,
            R7 => R5,
        }
    }

    /// # Safety
    /// This is safe as long as value is an even permutation
    pub const unsafe fn from_permutation_unchecked(value: GenericPermutation<4>) -> Self {
        use A4::*;
        match value.as_array() {
            [0, 1, 2, 3] => E,
            [1, 0, 3, 2] => U1,
            [3, 2, 1, 0] => U2,
            [2, 3, 0, 1] => U3,
            [0, 2, 3, 1] => R6,
            [3, 1, 0, 2] => R3,
            [2, 0, 1, 3] => R2,
            [1, 3, 2, 0] => R7,
            [3, 0, 2, 1] => R5,
            [1, 2, 0, 3] => R1,
            [2, 1, 3, 0] => R8,
            [0, 3, 1, 2] => R4,
            _ => std::hint::unreachable_unchecked(),
        }
    }

    pub const fn to_permutation(self) -> GenericPermutation<4> {
        use A4::*;
        // SAFTEY: These are all valid permutations
        unsafe {
            GenericPermutation::from_array_unchecked(match self {
                E => [0, 1, 2, 3],
                U1 => [1, 0, 3, 2],
                U2 => [3, 2, 1, 0],
                U3 => [2, 3, 0, 1],
                R6 => [0, 2, 3, 1],
                R3 => [3, 1, 0, 2],
                R2 => [2, 0, 1, 3],
                R7 => [1, 3, 2, 0],
                R5 => [3, 0, 2, 1],
                R1 => [1, 2, 0, 3],
                R8 => [2, 1, 3, 0],
                R4 => [0, 3, 1, 2],
            })
        }
    }

    pub const fn to_c3(self) -> super::C3 {
        use A4::*;
        match self {
            E | U1 | U2 | U3 => C3::E,
            R1 | R8 | R5 | R4 => C3::A,
            R2 | R6 | R3 | R7 => C3::AA,
        }
    }

    pub const fn to_k4(self) -> super::K4 {
        use A4::*;
        match self {
            E | R1 | R2 => K4::E,
            U1 | R8 | R6 => K4::U1,
            U2 | R5 | R3 => K4::U2,
            U3 | R4 | R7 => K4::U3,
        }
    }
}

impl Mul for A4 {
    type Output = A4;
    fn mul(self, rhs: Self) -> Self::Output {
        A4::group_mul(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_multiplication() {
        assert_eq!(A4::U1 * A4::U2, A4::U3);
        assert_eq!(A4::R6 * A4::R6, A4::R4);
        assert_eq!(A4::U1 * A4::R4, A4::R5);
        assert_eq!(A4::R2 * A4::R5, A4::U3);
        assert_eq!(A4::U3 * A4::R3, A4::R6);
        assert_eq!(A4::E * A4::R8, A4::R8);
    }
}
