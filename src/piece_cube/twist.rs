use std::{collections::HashMap, error::Error, str::FromStr};

use itertools::iproduct;
use lazy_static::lazy_static;
use num_enum::{FromPrimitive, TryFromPrimitive};
use strum::{EnumIter, IntoEnumIterator};
use strum_macros::EnumString;

use crate::{
    common::{Axis, Face, Sign, Vector3, Vector4},
    cubie_cube::{Move, HYPERSOLVE_TWISTS},
};

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum LayerEnum {
    This = 1,
    Other = 2,
    Both = 3,
}

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

impl std::fmt::Display for Twist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_mc4d_string())
    }
}

impl FromStr for Twist {
    type Err = TwistParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Twist::from_mc4d_twist_string(s)
    }
}

impl Twist {
    pub const fn axis(&self) -> Axis {
        self.face.axis()
    }

    pub const fn sign(&self) -> Sign {
        self.face.sign()
    }

    pub fn is_cube_rotation(&self) -> bool {
        self.layer == LayerEnum::Both
    }

    pub const fn new(face: Face, direction: TwistDirectionEnum, layer: LayerEnum) -> Twist {
        Twist {
            face,
            direction,
            layer,
        }
    }

    pub fn from_mc4d_twist_string(s: &str) -> Result<Twist, TwistParseError> {
        use TwistParseError::*;

        lazy_static! {
            static ref MC4D_TWISTS: Vec<Option<(Face, TwistDirectionEnum)>> =
                Twist::mc4d_twist_order().collect();
        }

        let mut segments = s.split(',');

        let twist_id_string = segments.next().ok_or(MissingTwistId)?.to_owned();

        let twist_id = twist_id_string
            .parse::<usize>()
            .or(Err(InvalidTwistId(twist_id_string.clone())))?;

        let (face, direction) = MC4D_TWISTS
            .get(twist_id)
            .ok_or(InvalidTwistId(twist_id_string.clone()))?
            .ok_or(InvalidTwistId(twist_id_string.clone()))?;

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

    pub fn to_mc4d_string(mut self: Twist) -> String {
        lazy_static! {
            static ref MC4D_TWIST_IDS: HashMap<(Face, TwistDirectionEnum), usize> =
                Twist::mc4d_twist_order()
                    .enumerate()
                    .filter_map(|(i, twist)| Some((twist?, i)))
                    .collect();
        }

        let dir: TwistDirectionEnum = self.direction;
        if let Some(quarter_turn) = dir.half() {
            self.direction = quarter_turn.into();
            return format!("{0} {0}", Self::to_mc4d_string(self));
        }
        let sticker_id = *MC4D_TWIST_IDS.get(&(self.face, self.direction)).unwrap();
        let direction_id = 1;
        let layer_mask = self.layer as u8;
        format!("{sticker_id},{direction_id},{layer_mask}")
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

    pub fn iter_all_twists() -> impl Iterator<Item = Twist> {
        iproduct!(Face::iter(), TwistDirectionEnum::iter())
            .map(|(face, direction)| Twist::new(face, direction, LayerEnum::This))
    }
}

// From Hyperspeedcube
// https://github.com/HactarCE/Hyperspeedcube/blob/645bbd3e88eec62d25a22c835a7174a0b2f44f99/src/piecepuzzle/common.rs
#[derive(FromPrimitive, Debug, Default, Copy, Clone, PartialEq, Eq, Hash, EnumIter, EnumString)]
#[repr(u8)]
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
}
