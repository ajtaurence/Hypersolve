use super::*;

use std::fmt::Display;

/// Faces of the hypercube
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    strum::EnumIter,
    strum::VariantArray,
    strum::EnumCount,
    Hash,
    const_gen::CompileConst,
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

impl Display for Face {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol_upper_str())
    }
}

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
    /// The axis this face is on
    pub const fn axis(self) -> Axis {
        use Face::*;
        match self {
            R | L => Axis::X,
            U | D => Axis::Y,
            F | B => Axis::Z,
            O | I => Axis::W,
        }
    }

    /// The sign of the axis this face is on
    pub const fn sign(self) -> Sign {
        use Face::*;
        match self {
            R | U | F | O => Sign::Pos,
            L | D | B | I => Sign::Neg,
        }
    }

    /// Get the face from an axis and the sign of the axis
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

    /// Get the opposite face
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

    /// Get the uppercase symbol for the face
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

    /// Returns the face from its uppercase symbol
    pub fn from_symbol_upper_str(s: &str) -> Option<Self> {
        match s {
            "R" => Some(Face::R),
            "L" => Some(Face::L),
            "U" => Some(Face::U),
            "D" => Some(Face::D),
            "F" => Some(Face::F),
            "B" => Some(Face::B),
            "O" => Some(Face::O),
            "I" => Some(Face::I),
            _ => None,
        }
    }

    pub const fn basis_faces(self) -> [Face; 3] {
        use Axis::*;

        let w = match self.sign() {
            Sign::Pos => Face::O,
            Sign::Neg => Face::I,
        };

        [
            if self.axis() as u8 == X as u8 {
                w
            } else {
                Face::R
            },
            if self.axis() as u8 == Y as u8 {
                w
            } else {
                Face::U
            },
            if self.axis() as u8 == Z as u8 {
                w
            } else {
                Face::F
            },
        ]
    }

    pub const fn basis(self) -> [[i32; 4]; 3] {
        let basis_faces = self.basis_faces();

        const_arr!([[i32; 4]; 3], |i| basis_faces[i].into_vector4())
    }

    pub const fn into_vector4(self) -> [i32; 4] {
        use Face::*;
        match self {
            R => [1, 0, 0, 0],
            U => [0, 1, 0, 0],
            F => [0, 0, 1, 0],
            O => [0, 0, 0, 1],
            L => [-1, 0, 0, 0],
            D => [0, -1, 0, 0],
            B => [0, 0, -1, 0],
            I => [0, 0, 0, -1],
        }
    }
}
