use strum::EnumCount;

use super::*;

/// An enum representing the axes X, Y, Z, and W
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    strum_macros::EnumIter,
    strum_macros::EnumCount,
    Hash,
)]
#[repr(u8)]
pub enum Axis {
    #[default]
    X = 0,
    Y = 1,
    Z = 2,
    W = 3,
}

impl Index for Axis {
    const NUM_INDICES: u64 = Axis::COUNT as u64;

    fn from_index(index: u64) -> Self {
        use Axis::*;
        match index {
            0 => X,
            1 => Y,
            2 => Z,
            3 => W,
            _ => unreachable!(),
        }
    }

    fn to_index(self) -> u64 {
        use Axis::*;
        match self {
            X => 0,
            Y => 1,
            Z => 2,
            W => 3,
        }
    }
}

impl std::fmt::Display for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Axis::*;
        let string = match self {
            X => "X",
            Y => "Y",
            Z => "Z",
            W => "W",
        };
        write!(f, "{}", string)
    }
}

macro_rules! axis_to_from_vector {
    ($type:ty) => {
        impl From<Axis> for Vector4<$type> {
            fn from(value: Axis) -> Self {
                use Axis::*;
                match value {
                    X => Vector([1, 0, 0, 0]),
                    Y => Vector([0, 1, 0, 0]),
                    Z => Vector([0, 0, 1, 0]),
                    W => Vector([0, 0, 0, 1]),
                }
            }
        }
        impl TryFrom<Vector4<$type>> for Axis {
            type Error = String;
            fn try_from(value: Vector4<$type>) -> Result<Self, Self::Error> {
                match value {
                    Vector([1, 0, 0, 0]) => Ok(Axis::X),
                    Vector([0, 1, 0, 0]) => Ok(Axis::Y),
                    Vector([0, 0, 1, 0]) => Ok(Axis::Z),
                    Vector([0, 0, 0, 1]) => Ok(Axis::W),
                    _ => Err(format!("Could not convert {:?} to an axis", value)),
                }
            }
        }
    };
}
for_each!(axis_to_from_vector!(
    i8, u8, i16, u16, i32, u32, i64, u64, i128, usize, isize
));

macro_rules! impl_axis_to_from_int {
    ($type:ty) => {
        impl From<Axis> for $type {
            fn from(value: Axis) -> $type {
                use Axis::*;
                match value {
                    X => 0,
                    Y => 1,
                    Z => 2,
                    W => 3,
                }
            }
        }

        impl TryFrom<$type> for Axis {
            type Error = String;
            fn try_from(value: $type) -> Result<Self, Self::Error> {
                use Axis::*;
                match value {
                    0 => Ok(X),
                    1 => Ok(Y),
                    2 => Ok(Z),
                    3 => Ok(W),
                    _ => Err(format!("cannot convert {:?} to a an axis", value)),
                }
            }
        }
    };
}
for_each!(impl_axis_to_from_int!(
    i8, u8, i16, u16, i32, u32, i64, u64, i128, usize, isize
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_to_from_int() {
        use strum::IntoEnumIterator;
        let to_from_int = |val: Axis| Axis::try_from(u32::from(val)).unwrap();

        for axis in Axis::iter() {
            assert_eq!(to_from_int(axis), axis);
        }
    }

    #[test]
    fn axis_to_from_vector() {
        use strum::IntoEnumIterator;
        let to_from_vector = |val: Axis| Axis::try_from(Vector4::<u32>::from(val)).unwrap();

        for axis in Axis::iter() {
            assert_eq!(to_from_vector(axis), axis);
        }
    }
}
