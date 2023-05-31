use super::*;

use strum::EnumCount;

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    strum_macros::EnumIter,
    strum_macros::EnumCount,
    strum_macros::EnumString,
)]
#[repr(u8)]
pub enum Face {
    R,
    L,
    U,
    D,
    F,
    B,
    O,
    I,
}

impl Index for Face {
    const NUM_INDICES: u64 = Face::COUNT as u64;

    fn from_index(index: u64) -> Self {
        use Face::*;
        match index {
            0 => R,
            1 => L,
            2 => U,
            3 => D,
            4 => F,
            5 => B,
            6 => O,
            7 => I,
            _ => unreachable!(),
        }
    }

    fn to_index(self) -> u64 {
        use Face::*;
        match self {
            R => 0,
            L => 1,
            U => 2,
            D => 3,
            F => 4,
            B => 5,
            O => 6,
            I => 7,
        }
    }
}

impl std::fmt::Display for Face {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol_upper_str())
    }
}

impl From<Face> for Sign {
    fn from(value: Face) -> Self {
        value.sign()
    }
}

impl From<Face> for Axis {
    fn from(value: Face) -> Self {
        value.axis()
    }
}

impl From<Axis> for Face {
    fn from(axis: Axis) -> Self {
        use Face::*;
        match axis {
            Axis::X => R,
            Axis::Y => U,
            Axis::Z => F,
            Axis::W => O,
        }
    }
}

macro_rules! vector_to_from_face {
    ($type:ty) => {
        impl From<Face> for Vector4<$type> {
            fn from(value: Face) -> Self {
                use Face::*;
                match value {
                    R => Vector([1, 0, 0, 0]),
                    U => Vector([0, 1, 0, 0]),
                    F => Vector([0, 0, 1, 0]),
                    O => Vector([0, 0, 0, 1]),
                    L => Vector([-1, 0, 0, 0]),
                    D => Vector([0, -1, 0, 0]),
                    B => Vector([0, 0, -1, 0]),
                    I => Vector([0, 0, 0, -1]),
                }
            }
        }
        impl TryFrom<Vector4<$type>> for Face {
            type Error = String;
            fn try_from(value: Vector4<$type>) -> Result<Self, Self::Error> {
                match value {
                    Vector([1, 0, 0, 0]) => Ok(Face::R),
                    Vector([0, 1, 0, 0]) => Ok(Face::F),
                    Vector([0, 0, 1, 0]) => Ok(Face::U),
                    Vector([0, 0, 0, 1]) => Ok(Face::O),
                    Vector([-1, 0, 0, 0]) => Ok(Face::L),
                    Vector([0, -1, 0, 0]) => Ok(Face::B),
                    Vector([0, 0, -1, 0]) => Ok(Face::D),
                    Vector([0, 0, 0, -1]) => Ok(Face::I),
                    _ => Err(format!("Could not convert {:?} to a face", value)),
                }
            }
        }
    };
}
for_each!(vector_to_from_face!(i8, i16, i32, i64, i128, isize));

impl std::ops::Mul<Sign> for Face {
    type Output = Self;
    fn mul(self, rhs: Sign) -> Self::Output {
        match rhs {
            Sign::Pos => self,
            Sign::Neg => self.opposite(),
        }
    }
}

impl std::ops::Neg for Face {
    type Output = Self;
    fn neg(self) -> Self::Output {
        self.opposite()
    }
}

impl Face {
    pub const fn axis(self) -> Axis {
        use Face::*;
        match self {
            R | L => Axis::X,
            U | D => Axis::Y,
            F | B => Axis::Z,
            O | I => Axis::W,
        }
    }

    pub const fn sign(self) -> Sign {
        use Face::*;
        match self {
            R | U | F | O => Sign::Pos,
            L | D | B | I => Sign::Neg,
        }
    }

    pub const fn from_axis_sign(axis: Axis, sign: Sign) -> Self {
        use Face::*;
        match (axis, sign) {
            (Axis::X, Sign::Pos) => R,
            (Axis::Y, Sign::Pos) => U,
            (Axis::Z, Sign::Pos) => F,
            (Axis::W, Sign::Pos) => O,
            (Axis::X, Sign::Neg) => L,
            (Axis::Y, Sign::Neg) => D,
            (Axis::Z, Sign::Neg) => B,
            (Axis::W, Sign::Neg) => I,
        }
    }

    pub const fn opposite(self) -> Face {
        use Face::*;
        match self {
            R => L,
            U => D,
            F => B,
            O => I,
            L => R,
            D => U,
            B => F,
            I => O,
        }
    }

    pub const fn symbol_upper_str(self) -> &'static str {
        match self {
            Face::R => "R",
            Face::L => "L",
            Face::U => "U",
            Face::D => "D",
            Face::F => "F",
            Face::B => "B",
            Face::O => "O",
            Face::I => "I",
        }
    }

    pub fn basis_faces(self) -> [Face; 3] {
        use Axis::*;

        let w = match self.sign() {
            Sign::Pos => Face::O,
            Sign::Neg => Face::I,
        };

        [
            if self.axis() == X { w } else { Face::R },
            if self.axis() == Y { w } else { Face::U },
            if self.axis() == Z { w } else { Face::F },
        ]
    }

    pub fn basis(self) -> [Vector4<i32>; 3] {
        self.basis_faces().map(|f| f.into())
    }
}
