use std::ops::Mul;

use enum_primitive_derive::Primitive;

use crate::puzzle::{AxisEnum, Piece};

pub trait Identity<T> {
    //Returns the identity
    fn identity() -> T;
}

// See http://escarbille.free.fr/group/?g=12_3a
#[derive(Debug, Default, Copy, Clone, PartialEq, Primitive)]
#[repr(u8)]
pub enum A4Elem {
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

impl Identity<A4Elem> for A4Elem {
    fn identity() -> A4Elem {
        A4Elem::E
    }
}

// Indexed as [right, left]
const A4_MUL_TABLE: [[A4Elem; 12]; 12] = {
    use A4Elem::*;
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

impl Mul<A4Elem> for A4Elem {
    type Output = A4Elem;

    #[inline]
    fn mul(self, rhs: A4Elem) -> A4Elem {
        A4_MUL_TABLE[rhs as usize][self as usize]
    }
}

// See http://escarbille.free.fr/group/?g=4_2a
#[derive(Debug, Default, Copy, Clone, PartialEq, Primitive)]
#[repr(u8)]
pub enum K4Elem {
    #[default]
    E = 0,
    U1 = 1,
    U2 = 2,
    U3 = 3,
}

impl Identity<K4Elem> for K4Elem {
    fn identity() -> K4Elem {
        K4Elem::E
    }
}

// Indexed as [right, left]
const K4_A4_MUL_TABLE: [[K4Elem; 4]; 12] = {
    use K4Elem::*;
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

impl Mul<A4Elem> for K4Elem {
    type Output = K4Elem;

    #[inline]
    fn mul(self, rhs: A4Elem) -> K4Elem {
        K4_A4_MUL_TABLE[rhs as usize][self as usize]
    }
}

impl From<A4Elem> for K4Elem {
    #[inline]
    fn from(value: A4Elem) -> Self {
        use A4Elem::*;

        match value {
            E | R1 | R2 => K4Elem::E,
            U1 | R8 | R6 => K4Elem::U1,
            U2 | R5 | R3 => K4Elem::U2,
            U3 | R4 | R7 => K4Elem::U3,
        }
    }
}

// See http://escarbille.free.fr/group/?g=3_1
#[derive(Debug, Default, Copy, Clone, Primitive)]
#[repr(u8)]
pub enum C3Elem {
    #[default]
    E = 0,
    A = 1,
    AA = 2,
}

impl Identity<C3Elem> for C3Elem {
    fn identity() -> C3Elem {
        C3Elem::E
    }
}

impl From<A4Elem> for C3Elem {
    #[inline]
    fn from(value: A4Elem) -> Self {
        use A4Elem::*;
        match value {
            E | U1 | U2 | U3 => C3Elem::E,
            R1 | R8 | R5 | R4 => C3Elem::A,
            R2 | R6 | R3 | R7 => C3Elem::AA,
        }
    }
}

impl Mul<C3Elem> for C3Elem {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: C3Elem) -> Self::Output {
        use C3Elem::*;
        match (self, rhs) {
            (E, val) | (val, E) => val,
            (AA, AA) => A,
            (A, A) => AA,
            (AA, A) | (A, AA) => E,
        }
    }
}

impl Mul<A4Elem> for C3Elem {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: A4Elem) -> Self::Output {
        self * C3Elem::from(rhs)
    }
}
