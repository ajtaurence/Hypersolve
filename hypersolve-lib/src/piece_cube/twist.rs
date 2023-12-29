use crate::{
    common::*,
    cubie_cube::{Move, HYPERSOLVE_TWISTS},
};
use itertools::Itertools;
use num_enum::{FromPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use std::{collections::HashMap, error::Error, str::FromStr};
use strum::{EnumIter, IntoEnumIterator};
use strum_macros::Display;

/// Notation types for twists
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Display)]
#[allow(clippy::upper_case_acronyms)]
pub enum Notation {
    #[default]
    Standard,
    MC4D,
}

/// Errors for parsing MC4D twist format
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TwistParseError {
    MissingTwistId,
    InvalidTwistId(String),
    MissingTwistAmount,
    InvalidTwistAmount(String),
    MissingTwistSliceMask,
    InvalidTwistSliceMask(String),
    UnexpectedValue(String),
}

impl Error for TwistParseError {}

impl std::fmt::Display for TwistParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            TwistParseError::MissingTwistId => "missing twist id".to_owned(),
            TwistParseError::InvalidTwistId(id) => format!("invalid twist id: {}", id),
            TwistParseError::MissingTwistAmount => "missing twist amount".to_owned(),
            TwistParseError::InvalidTwistAmount(amount) => {
                format!("invalid twist amount: {}", amount)
            }
            TwistParseError::MissingTwistSliceMask => "missing twist slice mask".to_owned(),
            TwistParseError::InvalidTwistSliceMask(mask) => {
                format!("invalid twist slice mask: {}", mask)
            }
            TwistParseError::UnexpectedValue(value) => {
                format!("unexpected continuation: ,{}", value)
            }
        };
        write!(f, "{}", string)
    }
}

/// Enum representing the layer grabbed when twisting
#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum LayerEnum {
    This = 1,
    Other = 2,
    Both = 3,
}

impl LayerEnum {
    fn to_standard_string(self) -> &'static str {
        match self {
            Self::This => "",
            Self::Other => "{2}",
            Self::Both => "{1-2}",
        }
    }
}

/// A twist that can be applied to the cube
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Twist {
    pub face: Face,
    pub direction: TwistDirectionEnum,
    pub layer: LayerEnum,
}

impl From<Move> for Twist {
    fn from(value: Move) -> Self {
        HYPERSOLVE_TWISTS[value.0 as usize]
    }
}

impl FromStr for Twist {
    type Err = TwistParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Twist::from_mc4d_twist_string(s)
    }
}

impl Twist {
    /// Creates a new twist
    pub const fn new(face: Face, direction: TwistDirectionEnum, layer: LayerEnum) -> Twist {
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

    /// Returns the string for this move in the given notation
    pub fn to_string(&self, notation: Notation) -> String {
        match notation {
            Notation::Standard => self.to_standard_string(),
            Notation::MC4D => self.to_mc4d_string(),
        }
    }

    /// Returns the inverse of this twist
    pub fn inverse(&self) -> Self {
        Self::new(self.face, self.direction.rev(), self.layer)
    }

    /// Returns whether this twist is a cube rotation
    pub fn is_cube_rotation(&self) -> bool {
        self.layer == LayerEnum::Both
    }

    /// Creates a twist from its MC4D string
    pub fn from_mc4d_twist_string(s: &str) -> Result<Twist, TwistParseError> {
        use TwistParseError::*;

        static MC4D_TWISTS: Lazy<Vec<Option<(Face, TwistDirectionEnum)>>> =
            Lazy::new(|| Twist::mc4d_twist_order().collect());

        let mut segments = s.split(',');

        let twist_id_string = segments.next().ok_or(MissingTwistId)?.to_owned();

        let twist_id = twist_id_string
            .parse::<usize>()
            .or(Err(InvalidTwistId(twist_id_string.clone())))?;

        let (face, direction) = MC4D_TWISTS
            .get(twist_id)
            .ok_or(InvalidTwistId(twist_id_string.clone()))?
            .ok_or(InvalidTwistId(twist_id_string))?;

        let twist_amount_string = segments.next().ok_or(MissingTwistAmount)?.to_owned();

        let direction = match twist_amount_string
            .parse::<i8>()
            .or(Err(InvalidTwistAmount(twist_amount_string.clone())))?
        {
            1 => direction,
            2 => direction
                .double()
                .ok_or(InvalidTwistAmount(twist_amount_string.clone()))?,
            -1 => direction.rev(),
            -2 => direction
                .rev()
                .double()
                .ok_or(InvalidTwistAmount(twist_amount_string.clone()))?,
            _ => return Err(TwistParseError::InvalidTwistAmount(twist_amount_string)),
        };

        let slice_mask_string = segments.next().ok_or(MissingTwistSliceMask)?.to_owned();

        let layer = LayerEnum::try_from_primitive(
            slice_mask_string
                .parse()
                .or(Err(InvalidTwistSliceMask(slice_mask_string.clone())))?,
        )
        .or(Err(InvalidTwistSliceMask(slice_mask_string)))?;

        let next_string = segments.next();
        if let Some(value) = next_string {
            return Err(UnexpectedValue(value.to_owned()));
        }
        Ok(Twist::new(face, direction, layer))
    }

    /// Returns the MC4D string for this twist
    pub fn to_mc4d_string(mut self: Twist) -> String {
        static MC4D_TWIST_IDS: Lazy<HashMap<(Face, TwistDirectionEnum), usize>> = Lazy::new(|| {
            Twist::mc4d_twist_order()
                .enumerate()
                .filter_map(|(i, twist)| Some((twist?, i)))
                .collect()
        });

        let dir: TwistDirectionEnum = self.direction;
        if let Some(quarter_turn) = dir.half() {
            self.direction = quarter_turn;
            return format!("{0} {0}", Self::to_mc4d_string(self));
        }
        let sticker_id = *MC4D_TWIST_IDS.get(&(self.face, self.direction)).unwrap();
        let direction_id = 1;
        let layer_mask = self.layer as u8;
        format!("{sticker_id},{direction_id},{layer_mask}")
    }

    /// Returns the standard string for this twist    
    pub fn to_standard_string(self) -> String {
        if self.layer == LayerEnum::Other {
            return Twist::new(self.face.opposite(), self.direction.rev(), LayerEnum::This)
                .to_standard_string();
        }

        let neighboring_faces = self
            .direction
            .signs_within_face()
            .into_iter()
            .zip(self.face.basis_faces())
            .filter_map(|(s, f)| match s {
                ZeroOrSign::Zero => None,
                ZeroOrSign::Pos => Some(f * Sign::Pos),
                ZeroOrSign::Neg => Some(f * Sign::Neg),
            })
            .sorted_by_key(|f| match f.axis() {
                Axis::Y => 0,
                Axis::Z => 1,
                Axis::X => 2,
                Axis::W => 3,
            });

        let faces = std::iter::once(self.face).chain(neighboring_faces);

        let mut string = self.layer.to_standard_string().to_owned();

        string.extend(faces.map(|f| f.to_string()));

        if self.direction.is_double() {
            string.push('2')
        }

        string
    }

    fn mc4d_twist_order() -> impl Iterator<Item = Option<(Face, TwistDirectionEnum)>> {
        const MC4D_FACE_ORDER: [Face; 8] = [
            Face::I,
            Face::B,
            Face::D,
            Face::L,
            Face::R,
            Face::U,
            Face::F,
            Face::O,
        ];

        let piece_locations =
            itertools::iproduct!([-1, 0, 1], [-1, 0, 1], [-1, 0, 1]).map(|(x, y, z)| [x, y, z]);
        let corners = piece_locations
            .clone()
            .filter(|v| Vector3::from(*v).magnitude_squared() == 3);
        let edges = piece_locations
            .clone()
            .filter(|v| Vector3::from(*v).magnitude_squared() == 2);
        let centers = piece_locations.filter(|v| Vector3::from(*v).magnitude_squared() == 1);
        let core = std::iter::once([0, 0, 0]);
        let mc4d_order_piece_locations = corners.chain(edges).chain(centers).chain(core);

        MC4D_FACE_ORDER.into_iter().flat_map(move |face| {
            let mut basis = face.basis_faces();
            basis.sort_by_key(|f| f.axis()); // order: X, Y, Z, W
            basis.reverse(); // order: W, Z, Y, X

            mc4d_order_piece_locations
                .clone()
                .map(move |mc4d_coords_of_sticker_within_face: [i32; 3]| {
                    let mut offset = Vector4::from_elem(0);
                    for i in 0..3 {
                        offset[basis[i].axis() as usize] += mc4d_coords_of_sticker_within_face[i];
                    }

                    TwistDirectionEnum::from_signs_within_face(Self::signs_within_face(
                        face,
                        match face {
                            Face::O => offset, // not sure why this is necessary, but it is
                            _ => -offset,
                        },
                    ))
                })
                .map(move |twist_dir| Some((face, twist_dir?)))
        })
    }

    fn signs_within_face(face: Face, piece_loc_signs: Vector4<i32>) -> Vector3<i32> {
        let [basis1, basis2, basis3] = face.basis();
        vector!(
            piece_loc_signs.dot(basis1),
            piece_loc_signs.dot(basis2),
            piece_loc_signs.dot(basis3)
        )
    }

    /// Iterates over all possible twists
    pub fn iter_all_twists() -> impl Iterator<Item = Twist> {
        itertools::iproduct!(Face::iter(), TwistDirectionEnum::iter())
            .map(|(face, direction)| Twist::new(face, direction, LayerEnum::This))
    }
}

/// 3D Twist directions from [Hyperspeedcube](https://github.com/HactarCE/Hyperspeedcube/blob/9ef4d7f7c4a273b4ffb723e65e4539593c156322/src/puzzle/rubiks_4d.rs#L967C1-L1039C2)
#[derive(
    FromPrimitive,
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    strum_macros::EnumString,
)]
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
pub enum TwistDirectionEnum {
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

impl TwistDirectionEnum {
    pub fn symbol_xyz(self) -> &'static str {
        use TwistDirectionEnum::*;

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

    fn rev(self) -> Self {
        Self::from(self as u8 ^ 1)
    }

    fn half(self) -> Option<Self> {
        use TwistDirectionEnum::*;

        match self {
            R2 | L2 | U2 | D2 | F2 | B2 => Some(Self::from(self as u8 - 6)),
            _ => None,
        }
    }

    fn is_double(self) -> bool {
        use TwistDirectionEnum::*;

        matches!(self, R2 | L2 | U2 | D2 | F2 | B2)
    }

    fn double(self) -> Option<Self> {
        use TwistDirectionEnum::*;

        match self {
            R | L | U | D | F | B => Some(Self::from(self as u8 + 6)),
            R2 | L2 | U2 | D2 | F2 | B2 => None,
            UF | DB | UR | DL | FR | BL | DF | UB | UL | DR | BR | FL => None,
            UFR | DBL | UFL | DBR | DFR | UBL | UBR | DFL => Some(self.rev()),
        }
    }

    fn from_signs_within_face(v: Vector3<i32>) -> Option<Self> {
        use TwistDirectionEnum::*;

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

    fn signs_within_face(&self) -> Vector<ZeroOrSign, 3> {
        use TwistDirectionEnum::*;

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

/// Wrapper on a vector of twists
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TwistSequence(pub Vec<Twist>);

impl From<Vec<Move>> for TwistSequence {
    fn from(value: Vec<Move>) -> Self {
        value.into_iter().map(|m| m.into()).collect()
    }
}

impl TwistSequence {
    /// Returns the inverse of this twist sequence
    pub fn inverse(&self) -> Self {
        self.0.iter().rev().map(|twist| twist.inverse()).collect()
    }

    pub fn to_string(&self, notation: Notation) -> String {
        self.0
            .iter()
            .map(|twist| twist.to_string(notation))
            .join(" ")
    }
}

impl FromStr for TwistSequence {
    type Err = TwistParseError;

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
        use crate::piece_cube::puzzle::PieceCube;
        let solved = PieceCube::solved();

        for face in Face::iter() {
            assert_eq!(
                solved
                    .twist(Twist {
                        face,
                        direction: TwistDirectionEnum::R,
                        layer: LayerEnum::This
                    })
                    .twist(Twist {
                        face,
                        direction: TwistDirectionEnum::U,
                        layer: LayerEnum::This
                    }),
                solved.twist(Twist {
                    face,
                    direction: TwistDirectionEnum::UFR,
                    layer: LayerEnum::This
                })
            );
        }
    }

    #[test]
    fn test_twist_standard_string() {
        let twist = Twist::new(Face::I, TwistDirectionEnum::R, LayerEnum::This);
        assert_eq!(twist.to_standard_string(), "IR");

        let twist = Twist::new(Face::I, TwistDirectionEnum::R, LayerEnum::Other);
        assert_eq!(twist.to_standard_string(), "OL");

        let twist = Twist::new(Face::R, TwistDirectionEnum::F, LayerEnum::This);
        assert_eq!(twist.to_standard_string(), "RF");

        let twist = Twist::new(Face::R, TwistDirectionEnum::F, LayerEnum::Both);
        assert_eq!(twist.to_standard_string(), "{1-2}RF");

        let twist = Twist::new(Face::B, TwistDirectionEnum::UFR, LayerEnum::This);
        assert_eq!(twist.to_standard_string(), "BURI");

        let twist = Twist::new(Face::R, TwistDirectionEnum::UFR, LayerEnum::This);
        assert_eq!(twist.to_standard_string(), "RUFO");
    }
}
