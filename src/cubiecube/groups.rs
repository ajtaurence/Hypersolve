use enum_primitive_derive::Primitive;
use itertools::Itertools;
use lazy_static::__Deref;
use std::{
    fmt::Display,
    ops::{DerefMut, Mul},
};
use strum_macros::EnumIter;

use crate::{math::lcm, piece_cube::pieces::Piece};

/// Even or odd parity
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Parity {
    Even,
    Odd,
}

impl Mul for Parity {
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

pub trait Identity {
    const IDENTITY: Self;
}

/// Permutation of elements in word form.
///
/// Multiplication takes the state on the right and acts on it with the state on the left.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PermutationWord<const N: usize>(pub [usize; N]);

impl<const N: usize> Display for PermutationWord<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<const N: usize> Default for PermutationWord<N> {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl<const N: usize> Identity for PermutationWord<N> {
    const IDENTITY: Self = {
        let mut result = [0; N];
        let mut i = 0;
        while i < N {
            result[i] = i;
            i += 1;
        }
        PermutationWord(result)
    };
}

impl<const N: usize> __Deref for PermutationWord<N> {
    type Target = [usize; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for PermutationWord<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> Mul for PermutationWord<N> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = [0; N];
        for i in 0..N {
            result[i] = rhs[self[i]]
        }
        PermutationWord(result)
    }
}

impl<const N: usize> PermutationWord<N> {
    /// Returns an iterator over all permutations
    pub fn iter_permutations() -> impl Iterator<Item = Self> {
        (0..N)
            .into_iter()
            .permutations(N)
            .map(|p| PermutationWord(p.try_into().unwrap()))
    }

    /// Returns the inverse of this permutation
    pub fn inverse(&self) -> Self {
        let mut result = [0; N];
        for i in 0..N {
            result[self[i]] = i;
        }
        PermutationWord(result)
    }

    /// Returns the parity of the permutation
    pub fn parity(&self) -> Parity {
        let mut visited = [false; N];
        let mut cycles = 0;

        for i in 0..N {
            if visited[i] {
                continue;
            }

            let mut current_index = self[i];
            visited[i] = true;
            while current_index != i {
                visited[current_index] = true;
                current_index = self[current_index];
            }
            cycles += 1;
        }
        match (-1_i32).pow((N - cycles) as u32) {
            1 => Parity::Even,
            -1 => Parity::Odd,
            _ => unreachable!(),
        }
    }

    /// Returns the order of the permutation
    pub fn order(&self) -> usize {
        let mut visited = [false; N];
        let mut cycle_lengths = Vec::new();

        for i in 0..N {
            if visited[i] {
                continue;
            }

            let mut current_index = self[i];
            let mut cycle_length = 1;
            visited[i] = true;
            while current_index != i {
                visited[current_index] = true;
                current_index = self[current_index];
                cycle_length += 1;
            }
            if cycle_length > 1 {
                cycle_lengths.push(cycle_length)
            }
        }

        cycle_lengths.into_iter().fold(1, |acc, i| lcm(acc, i))
    }

    /// Returns the conjugate of this permutation by another permutation
    pub fn conjugate(&self, other: &Self) -> Self {
        *other * *self * other.inverse()
    }

    // Returns a the permutation with entries at a and b swapped
    pub fn swap(&self, a: usize, b: usize) -> Self {
        assert!(
            a < N,
            "a: {} must be less than the number of elements {}",
            a,
            N
        );
        assert!(
            b < N,
            "b: {} must be less than the number of elements {}",
            b,
            N
        );
        let mut result = *self;
        let temp = result[a];
        result[a] = result[b];
        result[b] = temp;
        return result;
    }
}

/// Elements of the A4 group.
///
/// See http://escarbille.free.fr/group/?g=12_3a
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, EnumIter)]
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

impl Identity for A4 {
    const IDENTITY: Self = A4::E;
}

impl From<A4> for PermutationWord<4> {
    fn from(value: A4) -> Self {
        use A4::*;
        PermutationWord(match value {
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

impl TryFrom<PermutationWord<4>> for A4 {
    type Error = String;
    fn try_from(value: PermutationWord<4>) -> Result<Self, Self::Error> {
        use A4::*;
        match value.0 {
            [0, 1, 2, 3] => Ok(E),
            [1, 0, 3, 2] => Ok(U1),
            [3, 2, 1, 0] => Ok(U2),
            [2, 3, 0, 1] => Ok(U3),
            [0, 2, 3, 1] => Ok(R6),
            [3, 1, 0, 2] => Ok(R3),
            [2, 0, 1, 3] => Ok(R2),
            [1, 3, 2, 0] => Ok(R7),
            [3, 0, 2, 1] => Ok(R5),
            [1, 2, 0, 3] => Ok(R1),
            [2, 1, 3, 0] => Ok(R8),
            [0, 3, 1, 2] => Ok(R4),
            _ => Err(format!("{} is not an A4 group element", value)),
        }
    }
}

impl From<Piece> for A4 {
    fn from(piece: Piece) -> Self {
        // get the permutation of the axes of the piece
        let mut axis_permutation = piece.to_axis_permutation();

        // if the piece is at an odd location then swap the stickers on the X and Y axis
        // becase moves that shouldn't affect orientation swap the stickers on the X and Y axis
        if piece.current_location().parity().is_odd() {
            axis_permutation = axis_permutation.swap(0, 1);
        };

        // if we need to fix the parity of the piece to make it an A4 element then swap which
        // stickers are the X and Y stickers because we don't distinguish them for orientation purposes
        if axis_permutation.parity().is_odd() {
            axis_permutation = axis_permutation.inverse().swap(0, 1).inverse();
        }

        // convert the permutation to an A4 element
        A4::try_from(axis_permutation).unwrap()
    }
}

// Indexed as [left, right]
const_data!(
    A4_MUL_TABLE: [[A4; 12]; 12] = {
        use strum::IntoEnumIterator;
        let mut result = Box::new([[A4::E; 12]; 12]);

        for (elem1, elem2) in A4::iter().cartesian_product(A4::iter()) {
            let result_elem =
                A4::try_from(PermutationWord::from(elem1) * PermutationWord::from(elem2)).unwrap();

            result[elem1 as usize][elem2 as usize] = result_elem;
        }
        result
    }
);

/// Indexed as [left][right]
// const A4_MUL_TABLE: [[A4; 12]; 12] = {
//     use A4::*;
//     [
//         [E, R1, R2, U1, R8, R6, U2, R5, R3, U3, R4, R7],
//         [R1, R2, E, R4, R7, U3, R8, R6, U1, R5, R3, U2],
//         [R2, E, R1, R3, U2, R5, R7, U3, R4, R6, U1, R8],
//         [U1, R8, R6, E, R1, R2, U3, R4, R7, U2, R5, R3],
//         [R8, R6, U1, R5, R3, U2, R1, R2, E, R4, R7, U3],
//         [R6, U1, R8, R7, U3, R4, R3, U2, R5, R2, E, R1],
//         [U2, R5, R3, U3, R4, R7, E, R1, R2, U1, R8, R6],
//         [R5, R3, U2, R8, R6, U1, R4, R7, U3, R1, R2, E],
//         [R3, U2, R5, R2, E, R1, R6, U1, R8, R7, U3, R4],
//         [U3, R4, R7, U2, R5, R3, U1, R8, R6, E, R1, R2],
//         [R4, R7, U3, R1, R2, E, R5, R3, U2, R8, R6, U1],
//         [R7, U3, R4, R6, U1, R8, R2, E, R1, R3, U2, R5],
//     ]
// };

impl Mul for A4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        A4_MUL_TABLE[self as usize][rhs as usize]
    }
}

#[test]
fn test_a4_multiplication() {
    assert_eq!(A4::U1 * A4::U2, A4::U3);
    assert_eq!(A4::R6 * A4::R6, A4::R4);
    assert_eq!(A4::U1 * A4::R4, A4::R5);
    assert_eq!(A4::R2 * A4::R5, A4::U3);
    assert_eq!(A4::U3 * A4::R3, A4::R6);
    assert_eq!(A4::E * A4::R8, A4::R8);
}

/// Elements of the Klein group.
///
/// See http://escarbille.free.fr/group/?g=4_2a
#[derive(Debug, Default, Copy, Clone, PartialEq, Primitive)]
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

impl Identity for K4 {
    const IDENTITY: Self = K4::E;
}

impl From<A4> for K4 {
    fn from(value: A4) -> Self {
        use A4::*;

        match value {
            E | R1 | R2 => K4::E,
            U1 | R8 | R6 => K4::U1,
            U2 | R5 | R3 => K4::U2,
            U3 | R4 | R7 => K4::U3,
        }
    }
}
impl From<K4> for A4 {
    fn from(value: K4) -> Self {
        match value {
            K4::E => A4::E,
            K4::U1 => A4::U1,
            K4::U2 => A4::U2,
            K4::U3 => A4::U3,
        }
    }
}

// Indexed as [left, right]
const_data!(
    A4_K4_MUL_TABLE: [[K4; 4]; 12] = {
        use strum::IntoEnumIterator;
        let mut result = Box::new([[K4::E; 4]; 12]);

        for (elem1, elem2) in A4::iter().cartesian_product(A4::iter()) {
            let result_elem = K4::from(elem1 * elem2);

            // make sure when we overwrite a previously calculated value that it is the same
            let existing_value = result[elem1 as usize][K4::from(elem2) as usize];
            assert!(existing_value == result_elem || existing_value == K4::E);

            result[elem1 as usize][K4::from(elem2) as usize] = result_elem;
        }
        result
    }
);

impl Mul<K4> for A4 {
    type Output = K4;

    fn mul(self, rhs: K4) -> K4 {
        A4_K4_MUL_TABLE[self as usize][rhs as usize]
    }
}

/// Elements of the cyclic group of order 3.
///
/// See http://escarbille.free.fr/group/?g=3_1
#[derive(Debug, Default, Copy, Clone, Primitive, PartialEq, Eq)]
#[repr(u8)]
pub enum C3 {
    #[default]
    E = 0,
    A = 1,
    AA = 2,
}

impl Identity for C3 {
    const IDENTITY: Self = C3::E;
}

impl From<A4> for C3 {
    fn from(value: A4) -> Self {
        use A4::*;
        match value {
            E | U1 | U2 | U3 => C3::E,
            R1 | R8 | R5 | R4 => C3::A,
            R2 | R6 | R3 | R7 => C3::AA,
        }
    }
}
impl From<C3> for A4 {
    fn from(value: C3) -> Self {
        match value {
            C3::E => A4::E,
            C3::A => A4::R1,
            C3::AA => A4::R2,
        }
    }
}

impl Mul for C3 {
    type Output = Self;

    fn mul(self, rhs: C3) -> Self::Output {
        use C3::*;
        match (self, rhs) {
            (E, val) | (val, E) => val,
            (AA, AA) => A,
            (A, A) => AA,
            (AA, A) | (A, AA) => E,
        }
    }
}

impl Mul<A4> for C3 {
    type Output = Self;

    fn mul(self, rhs: A4) -> Self::Output {
        self * C3::from(rhs)
    }
}

impl From<C3> for PermutationWord<3> {
    fn from(value: C3) -> Self {
        use C3::*;
        PermutationWord(match value {
            E => [0, 1, 2],
            A => [1, 2, 0],
            AA => [2, 0, 1],
        })
    }
}

impl TryFrom<PermutationWord<3>> for C3 {
    type Error = String;
    fn try_from(value: PermutationWord<3>) -> Result<Self, Self::Error> {
        use C3::*;
        match value.0 {
            [0, 1, 2] => Ok(E),
            [1, 2, 0] => Ok(A),
            [2, 0, 1] => Ok(AA),
            _ => Err(format!("{} is not a C3 group element", value)),
        }
    }
}
