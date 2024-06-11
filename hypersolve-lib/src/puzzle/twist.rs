use strum::IntoEnumIterator;

use super::*;

/// Layer(s) grabbed when twisting
#[derive(Debug, Copy, Clone, PartialEq, Eq, num_enum::TryFromPrimitive, Hash)]
#[repr(u8)]
pub enum Layer {
    This = 1,
    Other = 2,
    Both = 3,
}

impl Layer {
    /// Returns the opposite of the selected layers
    pub const fn opposite(&self) -> Option<Self> {
        use Layer::*;
        match self {
            This => Some(Other),
            Other => Some(This),
            Both => None,
        }
    }
}

/// A twist that can be applied to a [`Cube`]
///
/// Twists are defined by a face, direction, and a layer:
/// * [`Face`] determines which face of the hypercube is gripped
/// * [`TwistDirection`] determines how the 3D slice is twisted
/// * [`Layer`] determines which layers are gripped
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Twist {
    pub face: Face,
    pub direction: TwistDirection,
    pub layer: Layer,
}

impl std::str::FromStr for Twist {
    type Err = ParseTwistError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let notation = if s
            .chars()
            .all(|c| c.is_ascii_punctuation() || c.is_ascii_digit())
        {
            // string is likely in MC4D notation
            Notation::MC4D
        } else if s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c.is_ascii_punctuation())
        {
            // string is likely in standard notation
            Notation::Standard
        } else {
            // unknown notation
            return Err(ParseTwistError::UnrecognizedNotation(s.into()));
        };

        notation.parse_twist(s)
    }
}

impl Twist {
    /// Creates a new twist
    pub const fn new(face: Face, direction: TwistDirection, layer: Layer) -> Twist {
        Twist {
            face,
            direction,
            layer,
        }
    }

    /// Returns the face this twist is on
    pub const fn face(&self) -> Face {
        self.face
    }

    /// Returns the axis this twist is on
    pub const fn axis(&self) -> Axis {
        self.face.axis()
    }

    /// Returns the sign of the axis this twist is on
    pub const fn sign(&self) -> Sign {
        self.face.sign()
    }

    /// Returns the twist that undoes this twist
    ///
    /// Not to be confused with [`reverse()`](#method.reverse)
    pub const fn inverse(&self) -> Self {
        Self::new(self.face, self.direction.reverse(), self.layer)
    }

    /// Returns the twist with the opposite layer mask
    ///
    /// Not to be confused with [`inverse()`](#method.inverse)
    pub const fn reverse(&self) -> Option<Self> {
        if let Some(new_layer) = self.layer.opposite() {
            Some(Self::new(self.face, self.direction, new_layer))
        } else {
            None
        }
    }

    /// Returns whether this twist is a cube rotation
    pub const fn is_cube_rotation(&self) -> bool {
        self.layer as u8 == Layer::Both as u8
    }

    /// Returns the string for this move in the given notation
    pub fn to_notation(&self, notation: Notation) -> String {
        notation.format_twist(self)
    }

    /// Iterates over all possible twists (excluding cube rotations)
    pub fn iter() -> impl Iterator<Item = Twist> {
        itertools::iproduct!(Face::iter(), TwistDirection::iter())
            .map(|(face, direction)| Twist::new(face, direction, Layer::This))
    }
}

/// 3D Twist directions
///
/// Taken from [Hyperspeedcube](https://github.com/HactarCE/Hyperspeedcube/blob/9ef4d7f7c4a273b4ffb723e65e4539593c156322/src/puzzle/rubiks_4d.rs#L967C1-L1039C2)
/// with some modifications
#[derive(
    num_enum::FromPrimitive,
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    strum::EnumIter,
    strum_macros::EnumString,
)]
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
pub enum TwistDirection {
    /// 90-degree face (2c) twist clockwise around `R`
    #[default]
    R,
    /// 90-degree face (2c) twist clockwise around `L`
    L,
    /// 90-degree face (2c) twist clockwise around `U`
    U,
    /// 90-degree face (2c) twist clockwise around `D`
    D,
    /// 90-degree face (2c) twist clockwise around `F`
    F,
    /// 90-degree face (2c) twist clockwise around `B`
    B,

    /// 180-degree face (2c) twist clockwise around `R`
    R2,
    /// 180-degree face (2c) twist clockwise around `L`
    L2,
    /// 180-degree face (2c) twist clockwise around `U`
    U2,
    /// 180-degree face (2c) twist clockwise around `D`
    D2,
    /// 180-degree face (2c) twist clockwise around `F`
    F2,
    /// 180-degree face (2c) twist clockwise around `B`
    B2,

    /// 180-degree edge (3c) twist clockwise around `UF`
    UF,
    /// 180-degree edge (3c) twist clockwise around `DB`
    DB,
    /// 180-degree edge (3c) twist clockwise around `UR`
    UR,
    /// 180-degree edge (3c) twist clockwise around `DL`
    DL,
    /// 180-degree edge (3c) twist clockwise around `FR`
    FR,
    /// 180-degree edge (3c) twist clockwise around `BL`
    BL,
    /// 180-degree edge (3c) twist clockwise around `DF`
    DF,
    /// 180-degree edge (3c) twist clockwise around `UB`
    UB,
    /// 180-degree edge (3c) twist clockwise around `UL`
    UL,
    /// 180-degree edge (3c) twist clockwise around `DR`
    DR,
    /// 180-degree edge (3c) twist clockwise around `BR`
    BR,
    /// 180-degree edge (3c) twist clockwise around `FL`
    FL,

    /// 120-degree corner (4c) twist clockwise around `UFR`
    UFR,
    /// 120-degree corner (4c) twist clockwise around `DBL`
    DBL,
    /// 120-degree corner (4c) twist clockwise around `UFL`
    UFL,
    /// 120-degree corner (4c) twist clockwise around `DBR` (equivalent: z'x)
    DBR,
    /// 120-degree corner (4c) twist clockwise around `DFR`
    DFR,
    /// 120-degree corner (4c) twist clockwise around `UBL` (equivalent: z'y)
    UBL,
    /// 120-degree corner (4c) twist clockwise around `UBR`
    UBR,
    /// 120-degree corner (4c) twist clockwise around `DFL` (equivalent: y'z)
    DFL,
}

impl TwistDirection {
    pub(crate) fn symbol_xyz(self) -> &'static str {
        use TwistDirection::*;

        match self {
            R => "x",
            L => "x'",
            U => "y",
            D => "y'",
            F => "z",
            B => "z'",

            R2 => "x2",
            L2 => "x2'",
            U2 => "y2",
            D2 => "y2'",
            F2 => "z2",
            B2 => "z2'",

            UF => "xy2",
            DB => "xy2'",
            UR => "zx2",
            DL => "zx2'",
            FR => "yz2",
            BL => "yz2'",
            DF => "xz2",
            UB => "xz2'",
            UL => "zy2",
            DR => "zy2'",
            BR => "yx2",
            FL => "yx2'",

            UFR => "xy",
            DBL => "y'x'",
            UFL => "zy",
            DBR => "xy'", // (equivalent: z'x)
            DFR => "xz",
            UBL => "yz'", // (equivalent: z'y)
            UBR => "yx",
            DFL => "zx'", // (equivalent: y'z)
        }
    }

    /// Returns the opposite twist direction
    pub const fn reverse(self) -> Self {
        use TwistDirection::*;

        match self {
            R => L,
            L => R,
            U => D,
            D => U,
            F => B,
            B => F,
            R2 => L2,
            L2 => R2,
            U2 => D2,
            D2 => U2,
            F2 => B2,
            B2 => F2,
            UF => DB,
            DB => UF,
            UR => DL,
            DL => UR,
            FR => BL,
            BL => FR,
            DF => UB,
            UB => DF,
            UL => DR,
            DR => UL,
            BR => FL,
            FL => BR,
            UFR => DBL,
            DBL => UFR,
            UFL => DBR,
            DBR => UFL,
            DFR => UBL,
            UBL => DFR,
            UBR => DFL,
            DFL => UBR,
        }
    }

    /// Returns half of the twist direction if possible
    pub const fn half(self) -> Option<Self> {
        use TwistDirection::*;

        match self {
            R2 => Some(R),
            L2 => Some(L),
            U2 => Some(U),
            D2 => Some(D),
            F2 => Some(F),
            B2 => Some(B),
            _ => None,
        }
    }

    /// Returns whether the twist direction is a dobule twist direction
    pub const fn is_double(self) -> bool {
        use TwistDirection::*;

        matches!(self, R2 | L2 | U2 | D2 | F2 | B2)
    }

    /// Returns the twist direction equivalent to performing this twist direction twice
    /// or `None` if the resulting twist direction is to do nothing
    pub const fn double(self) -> Option<Self> {
        use TwistDirection::*;

        match self {
            R => Some(R2),
            L => Some(L2),
            U => Some(U2),
            D => Some(D2),
            F => Some(F2),
            B => Some(B2),
            R2 | L2 | U2 | D2 | F2 | B2 | UF | DB | UR | DL | FR | BL | DF | UB | UL | DR | BR
            | FL => None,
            UFR | DBL | UFL | DBR | DFR | UBL | UBR | DFL => Some(self.reverse()),
        }
    }

    pub(crate) const fn from_signs_within_face(v: Vector3<i32>) -> Option<Self> {
        use TwistDirection::*;

        match v.0 {
            [1, 1, 1] => Some(UFR),
            [-1, 1, 1] => Some(UFL),
            [1, -1, 1] => Some(DFR),
            [-1, -1, 1] => Some(DFL),
            [1, 1, -1] => Some(UBR),
            [-1, 1, -1] => Some(UBL),
            [1, -1, -1] => Some(DBR),
            [-1, -1, -1] => Some(DBL),

            [1, 1, 0] => Some(UR),
            [-1, 1, 0] => Some(UL),
            [1, -1, 0] => Some(DR),
            [-1, -1, 0] => Some(DL),
            [1, 0, 1] => Some(FR),
            [-1, 0, 1] => Some(FL),
            [1, 0, -1] => Some(BR),
            [-1, 0, -1] => Some(BL),
            [0, 1, 1] => Some(UF),
            [0, -1, 1] => Some(DF),
            [0, 1, -1] => Some(UB),
            [0, -1, -1] => Some(DB),

            [1, 0, 0] => Some(R),
            [-1, 0, 0] => Some(L),
            [0, 1, 0] => Some(U),
            [0, -1, 0] => Some(D),
            [0, 0, 1] => Some(F),
            [0, 0, -1] => Some(B),

            _ => None,
        }
    }

    pub(crate) fn signs_within_face(&self) -> Vector<ZeroOrSign, 3> {
        use TwistDirection::*;

        Vector(match self.half().unwrap_or(*self) {
            UFR => [1, 1, 1],
            UFL => [-1, 1, 1],
            DFR => [1, -1, 1],
            DFL => [-1, -1, 1],
            UBR => [1, 1, -1],
            UBL => [-1, 1, -1],
            DBR => [1, -1, -1],
            DBL => [-1, -1, -1],

            UR => [1, 1, 0],
            UL => [-1, 1, 0],
            DR => [1, -1, 0],
            DL => [-1, -1, 0],
            FR => [1, 0, 1],
            FL => [-1, 0, 1],
            BR => [1, 0, -1],
            BL => [-1, 0, -1],
            UF => [0, 1, 1],
            DF => [0, -1, 1],
            UB => [0, 1, -1],
            DB => [0, -1, -1],

            R => [1, 0, 0],
            L => [-1, 0, 0],
            U => [0, 1, 0],
            D => [0, -1, 0],
            F => [0, 0, 1],
            B => [0, 0, -1],

            _ => unreachable!(),
        })
        .map(|i| i.try_into().unwrap())
    }
}

/// A sequence of consecutive twists
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TwistSequence(pub Vec<Twist>);

impl TwistSequence {
    /// Returns the inverse of this twist sequence
    pub fn inverse(&self) -> Self {
        self.0.iter().rev().map(|twist| twist.inverse()).collect()
    }
}

impl std::str::FromStr for TwistSequence {
    type Err = ParseTwistError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let twists = s.split_whitespace();

        let mut result = Vec::new();

        for twist_string in twists {
            result.push(Twist::from_str(twist_string)?)
        }

        Ok(TwistSequence(result))
    }
}

impl std::ops::Deref for TwistSequence {
    type Target = [Twist];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl std::ops::DerefMut for TwistSequence {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl IntoIterator for TwistSequence {
    type IntoIter = <Vec<Twist> as IntoIterator>::IntoIter;
    type Item = Twist;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Twist> for TwistSequence {
    fn from_iter<T: IntoIterator<Item = Twist>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twist_composition() {
        use crate::puzzle::Cube;
        let solved = Cube::solved();

        for face in Face::iter().filter(|f| *f != Face::O) {
            assert_eq!(
                solved
                    .twist(Twist {
                        face,
                        direction: TwistDirection::R,
                        layer: Layer::This
                    })
                    .twist(Twist {
                        face,
                        direction: TwistDirection::U,
                        layer: Layer::This
                    }),
                solved.twist(Twist {
                    face,
                    direction: TwistDirection::UFR,
                    layer: Layer::This
                })
            );
        }
    }
}
