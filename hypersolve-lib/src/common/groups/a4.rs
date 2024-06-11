use super::*;

/// Elements of the [A4 group](http://escarbille.free.fr/group/?g=12_3a)
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, strum::EnumIter)]
#[repr(u8)]
pub(crate) enum A4 {
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
    pub(crate) fn from_k4_c3(k4: K4, c3: C3) -> Self {
        // Make sure this is consistent with C3::from(A4)
        match (k4, c3) {
            (K4::E, c3_val) => Self::from(c3_val),
            (k4_val, C3::E) => Self::from(k4_val),
            (K4::U1, C3::A) => Self::R8,
            (K4::U1, C3::AA) => Self::R6,
            (K4::U2, C3::A) => Self::R5,
            (K4::U2, C3::AA) => Self::R3,
            (K4::U3, C3::A) => Self::R4,
            (K4::U3, C3::AA) => Self::R7,
        }
    }
}

impl From<A4> for Permutation<4> {
    fn from(value: A4) -> Self {
        use A4::*;
        Permutation::from_array_unchecked(match value {
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

impl TryFrom<Permutation<4>> for A4 {
    type Error = String;
    fn try_from(value: Permutation<4>) -> Result<Self, Self::Error> {
        use A4::*;
        match value.into_array() {
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

// Indexed as [left, right]
const_data!(
    A4_MUL_TABLE: [[A4; 12]; 12] = {
        use itertools::Itertools;
        use strum::IntoEnumIterator;
        let mut result = Box::new([[A4::E; 12]; 12]);

        for (elem1, elem2) in A4::iter().cartesian_product(A4::iter()) {
            let result_elem =
                A4::try_from(Permutation::from(elem1) * Permutation::from(elem2)).unwrap();

            result[elem1 as usize][elem2 as usize] = result_elem;
        }
        result
    }
);

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_a4_mul_table() {
    let _ = &*A4_MUL_TABLE;
}

impl BinaryOp for A4 {
    fn binary_op(a: Self, b: Self) -> Self {
        A4_MUL_TABLE[a as usize][b as usize]
    }
}

impl Identity for A4 {
    const IDENTITY: Self = A4::E;
}

impl std::ops::Mul for A4 {
    type Output = A4;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::binary_op(self, rhs)
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
