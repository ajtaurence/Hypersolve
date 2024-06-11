use super::*;

/// Elements of the [cyclic group of order 3](http://escarbille.free.fr/group/?g=3_1)
#[derive(
    Debug, Default, Copy, Clone, PartialEq, Eq, enum_primitive_derive::Primitive, strum::EnumIter,
)]
#[repr(u8)]
pub(crate) enum C3 {
    #[default]
    E = 0,
    A = 1,
    AA = 2,
}

impl Identity for C3 {
    const IDENTITY: Self = C3::E;
}

impl BinaryOp for C3 {
    fn binary_op(a: Self, b: Self) -> Self {
        use C3::*;
        match (a, b) {
            (E, val) | (val, E) => val,
            (AA, AA) => A,
            (A, A) => AA,
            (AA, A) | (A, AA) => E,
        }
    }
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

impl std::ops::Mul for C3 {
    type Output = Self;

    fn mul(self, rhs: C3) -> Self::Output {
        C3::binary_op(self, rhs)
    }
}

impl std::ops::Mul<A4> for C3 {
    type Output = Self;

    fn mul(self, rhs: A4) -> Self::Output {
        self * C3::from(rhs)
    }
}

impl From<C3> for Permutation<3> {
    fn from(value: C3) -> Self {
        use C3::*;
        Permutation::from_array_unchecked(match value {
            E => [0, 1, 2],
            A => [1, 2, 0],
            AA => [2, 0, 1],
        })
    }
}

impl TryFrom<Permutation<3>> for C3 {
    type Error = String;
    fn try_from(value: Permutation<3>) -> Result<Self, Self::Error> {
        use C3::*;
        match value.into_array() {
            [0, 1, 2] => Ok(E),
            [1, 2, 0] => Ok(A),
            [2, 0, 1] => Ok(AA),
            _ => Err(format!("{} is not a C3 group element", value)),
        }
    }
}
