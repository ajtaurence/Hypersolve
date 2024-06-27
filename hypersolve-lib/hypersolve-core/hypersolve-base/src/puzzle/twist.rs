use std::fmt::Debug;

use strum::{EnumCount, VariantArray};

use super::*;

/// Layer(s) grabbed when twisting
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    num_enum::TryFromPrimitive,
    Hash,
    strum::EnumCount,
    strum::VariantArray,
    const_gen::CompileConst,
)]
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
#[derive(Copy, Clone, PartialEq, Eq, const_gen::CompileConst)]
pub struct Twist {
    pub face: Face,
    pub direction: TwistDirection,
    pub layer: Layer,
}

impl Debug for Twist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Notation::Standard.format_twist(self))
    }
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
    /// All possible twists
    pub const ALL_TWISTS: [Twist; Face::COUNT * TwistDirection::COUNT] =
        const_arr!([Twist; Face::COUNT * TwistDirection::COUNT], |i| {
            let face_index = i / TwistDirection::COUNT;
            let twist_direction_index = i % TwistDirection::COUNT;

            Twist::new(
                Face::VARIANTS[face_index],
                TwistDirection::VARIANTS[twist_direction_index],
                Layer::This,
            )
        });

    /// Creates a new twist
    pub const fn new(face: Face, direction: TwistDirection, layer: Layer) -> Twist {
        Twist {
            face,
            direction,
            layer,
        }
    }

    /// Returns whether this twist affects the piece at a given location
    pub const fn affects_piece_at(&self, piece: PieceLocation) -> bool {
        piece.is_affected_by_twist(self)
    }

    /// Returns the face this twist is on
    pub const fn face(&self) -> Face {
        self.face
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
}

/// 3D Twist directions
///
/// Taken from [Hyperspeedcube](https://github.com/HactarCE/Hyperspeedcube) with some modifications
#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    strum::EnumString,
    strum::VariantArray,
    strum::EnumCount,
    strum::EnumIter,
    const_gen::CompileConst,
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
    pub const fn dirs_3d(self) -> &'static [(Axis, Sign)] {
        use Axis::*;
        use Sign::*;
        use TwistDirection::*;

        match self {
            R => &[(X, Pos)],
            L => &[(X, Neg)],
            U => &[(Y, Pos)],
            D => &[(Y, Neg)],
            F => &[(Z, Pos)],
            B => &[(Z, Neg)],

            R2 => &[(X, Pos), (X, Pos)],
            L2 => &[(X, Neg), (X, Neg)],
            U2 => &[(Y, Pos), (Y, Pos)],
            D2 => &[(Y, Neg), (Y, Neg)],
            F2 => &[(Z, Pos), (Z, Pos)],
            B2 => &[(Z, Neg), (Z, Neg)],

            UF => &[(X, Pos), (Y, Pos), (Y, Pos)],
            DB => &[(X, Pos), (Y, Neg), (Y, Neg)],
            UR => &[(Z, Pos), (X, Pos), (X, Pos)],
            DL => &[(Z, Pos), (X, Neg), (X, Neg)],
            FR => &[(Y, Pos), (Z, Pos), (Z, Pos)],
            BL => &[(Y, Pos), (Z, Neg), (Z, Neg)],
            DF => &[(X, Pos), (Z, Pos), (Z, Pos)],
            UB => &[(X, Pos), (Z, Neg), (Z, Neg)],
            UL => &[(Z, Pos), (Y, Pos), (Y, Pos)],
            DR => &[(Z, Pos), (Y, Neg), (Y, Neg)],
            BR => &[(Y, Pos), (X, Pos), (X, Pos)],
            FL => &[(Y, Pos), (X, Neg), (X, Neg)],

            UFR => &[(X, Pos), (Y, Pos)],
            DBL => &[(Y, Neg), (X, Neg)],
            UFL => &[(Z, Pos), (Y, Pos)],
            DBR => &[(X, Pos), (Y, Neg)], // (equivalent: z'x)
            DFR => &[(X, Pos), (Z, Pos)],
            UBL => &[(Y, Pos), (Z, Neg)], // (equivalent: z'y)
            UBR => &[(Y, Pos), (X, Pos)],
            DFL => &[(Z, Pos), (X, Neg)], // (equivalent: y'z)
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

    pub const fn from_signs_within_face(v: [i32; 3]) -> Option<Self> {
        use TwistDirection::*;

        match v {
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

    pub(crate) const fn signs_within_face(&self) -> [ZeroOrSign; 3] {
        use TwistDirection::*;
        use ZeroOrSign::*;

        let value = match self.half() {
            Some(val) => val,
            None => *self,
        };

        match value {
            UFR => [Pos, Pos, Pos],
            UFL => [Neg, Pos, Pos],
            DFR => [Pos, Neg, Pos],
            DFL => [Neg, Neg, Pos],
            UBR => [Pos, Pos, Neg],
            UBL => [Neg, Pos, Neg],
            DBR => [Pos, Neg, Neg],
            DBL => [Neg, Neg, Neg],

            UR => [Pos, Pos, Zero],
            UL => [Neg, Pos, Zero],
            DR => [Pos, Neg, Zero],
            DL => [Neg, Neg, Zero],
            FR => [Pos, Zero, Pos],
            FL => [Neg, Zero, Pos],
            BR => [Pos, Zero, Neg],
            BL => [Neg, Zero, Neg],
            UF => [Zero, Pos, Pos],
            DF => [Zero, Neg, Pos],
            UB => [Zero, Pos, Neg],
            DB => [Zero, Neg, Neg],

            R => [Pos, Zero, Zero],
            L => [Neg, Zero, Zero],
            U => [Zero, Pos, Zero],
            D => [Zero, Neg, Zero],
            F => [Zero, Zero, Pos],
            B => [Zero, Zero, Neg],

            _ => unsafe { std::hint::unreachable_unchecked() },
        }
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

    /// Returns the twist sequence as a string in the given notation
    pub fn to_notation(&self, notation: Notation) -> String {
        use itertools::Itertools;

        self.0
            .iter()
            .map(|twist| twist.to_notation(notation))
            .join(" ")
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
        use strum::IntoEnumIterator;

        let solved = Cube::SOLVED;

        for face in Face::iter().filter(|&f| f != Face::O) {
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

#[test]
fn test() {
    use itertools::Itertools;

    // Generate twist which dont affect LDBO (index 15) and perform unique actions on a cube
    let twists = Twist::ALL_TWISTS
        .into_iter()
        .filter(|twist| !PieceLocation::LAST.is_affected_by_twist(twist))
        .unique_by(|&twist| Cube::SOLVED.twist(twist))
        .collect_vec();

    println!("{:?}", twists);

    let twists = Twist::ALL_TWISTS
        .into_iter()
        .filter(|twist| !PieceLocation::LAST.is_affected_by_twist(twist))
        .collect_vec();

    println!("{:?}", twists)
}
