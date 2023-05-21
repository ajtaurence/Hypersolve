use super::*;

/// Elements of the Klein group.
///
/// See http://escarbille.free.fr/group/?g=4_2a
#[derive(
    Debug, Default, Copy, Clone, PartialEq, enum_primitive_derive::Primitive, strum_macros::EnumIter,
)]
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
        use itertools::Itertools;
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

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_a4_k4_mul_table() {
    let _ = &*A4_K4_MUL_TABLE;
}

impl std::ops::Mul<K4> for A4 {
    type Output = K4;

    fn mul(self, rhs: K4) -> K4 {
        A4_K4_MUL_TABLE[self as usize][rhs as usize]
    }
}

impl Identity for K4 {
    const IDENTITY: Self = K4::E;
}

impl BinaryOp for K4 {
    fn binary_op(a: Self, b: Self) -> Self {
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
}

impl Inverse for K4 {
    fn inverse(&self) -> Self {
        use K4::*;
        match self {
            E => E,
            U1 => U1,
            U2 => U2,
            U3 => U3,
        }
    }
}

impl Group for K4 {
    fn iter_elements() -> Box<dyn std::iter::Iterator<Item = Self>> {
        use strum::IntoEnumIterator;
        Box::new(K4::iter())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid_group() {
        assert!(K4::is_valid_group())
    }
}
