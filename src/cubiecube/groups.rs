use std::ops::Mul;

use enum_primitive_derive::Primitive;

use crate::piece_cube::pieces::{Axis, Piece};

pub trait Identity: Default {
    fn identity() -> Self {
        Self::default()
    }
}

/// Element of A4 group.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
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

impl Identity for A4 {}

impl From<Piece> for A4 {
    fn from(piece: Piece) -> Self {
        use Axis::*;
        match [
            piece.faces[0].axis(),
            piece.faces[1].axis(),
            piece.faces[2].axis(),
            piece.faces[3].axis(),
        ] {
            [_, _, Z, W] => A4::E,
            [_, _, W, Z] => A4::U1,
            [W, Z, _, _] => A4::U2,
            [Z, W, _, _] => A4::U3,
            [_, Z, W, _] => A4::R6,
            [_, W, Z, _] => A4::R7,
            [W, _, _, Z] => A4::R3,
            [Z, _, _, W] => A4::R2,
            [W, _, Z, _] => A4::R5,
            [Z, _, W, _] => A4::R8,
            [_, Z, _, W] => A4::R1,
            [_, W, _, Z] => A4::R4,
            _ => {
                unreachable!("Could not convert piece to A4 group element")
            }
        }
    }
}

/// Indexed as [right, left]
const A4_MUL_TABLE: [[A4; 12]; 12] = {
    use A4::*;
    [
        [E, R1, R2, U1, R8, R6, U2, R5, R3, U3, R4, R7],
        [R1, R2, E, R4, R7, U3, R8, R6, U1, R5, R3, U2],
        [R2, E, R1, R3, U2, R5, R7, U3, R4, R6, U1, R8],
        [U1, R8, R6, E, R1, R2, U3, R4, R7, U2, R5, R3],
        [R8, R6, U1, R5, R3, U2, R1, R2, E, R4, R7, U3],
        [R6, U1, R8, R7, U3, R4, R3, U2, R5, R2, E, R1],
        [U2, R5, R3, U3, R4, R7, E, R1, R2, U1, R8, R6],
        [R5, R3, U2, R8, R6, U1, R4, R7, U3, R1, R2, E],
        [R3, U2, R5, R2, E, R1, R6, U1, R8, R7, U3, R4],
        [U3, R4, R7, U2, R5, R3, U1, R8, R6, E, R1, R2],
        [R4, R7, U3, R1, R2, E, R5, R3, U2, R8, R6, U1],
        [R7, U3, R4, R6, U1, R8, R2, E, R1, R3, U2, R5],
    ]
};

impl Mul for A4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        A4_MUL_TABLE[rhs as usize][self as usize]
    }
}

/// See http://escarbille.free.fr/group/?g=4_2a
#[derive(Debug, Default, Copy, Clone, PartialEq, Primitive)]
#[repr(u8)]
pub enum K4 {
    #[default]
    E = 0,
    U1 = 1,
    U2 = 2,
    U3 = 3,
}

impl Identity for K4 {}

impl From<A4> for K4 {
    #[inline]
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

/// Indexed as [right, left]
const K4_A4_MUL_TABLE: [[K4; 4]; 12] = {
    use K4::*;
    [
        [E, U1, U2, U3],
        [E, U3, U1, U2],
        [E, U2, U3, U1],
        [U1, E, U3, U2],
        [U1, U2, E, U3],
        [U1, U3, U2, E],
        [U2, U3, E, U1],
        [U2, U1, U3, E],
        [U2, E, U1, U3],
        [U3, U2, U1, E],
        [U3, E, U2, U1],
        [U3, U1, E, U2],
    ]
};

impl Mul<A4> for K4 {
    type Output = K4;

    #[inline]
    fn mul(self, rhs: A4) -> K4 {
        K4_A4_MUL_TABLE[rhs as usize][self as usize]
    }
}

/// See http://escarbille.free.fr/group/?g=3_1
#[derive(Debug, Default, Copy, Clone, Primitive, PartialEq, Eq)]
#[repr(u8)]
pub enum C3 {
    #[default]
    E = 0,
    A = 1,
    AA = 2,
}

impl Identity for C3 {}

impl From<A4> for C3 {
    #[inline]
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

    #[inline]
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

    #[inline]
    fn mul(self, rhs: A4) -> Self::Output {
        self * C3::from(rhs)
    }
}
