use std::collections::HashMap;

use cgmath::{vec3, InnerSpace, Vector3, Vector4};
use itertools::iproduct;
use lazy_static::lazy_static;
use num_enum::FromPrimitive;
use strum::{EnumIter, IntoEnumIterator};

use super::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
pub enum LayerEnum {
    #[default]
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

impl Twist {
    pub const fn axis(&self) -> Axis {
        self.face.axis()
    }

    pub const fn sign(&self) -> Sign {
        self.face.sign()
    }

    const fn new(face: Face, direction: TwistDirectionEnum, layer: LayerEnum) -> Twist {
        Twist {
            face,
            direction,
            layer,
        }
    }

    pub fn from_mc4d_twist_string(s: &str) -> Option<Twist> {
        lazy_static! {
            static ref MC4D_TWISTS: Vec<Option<(Face, TwistDirectionEnum)>> =
                Twist::mc4d_twist_order().collect();
        }

        let mut segments = s.split(',');

        let (face, direction) = (*MC4D_TWISTS.get(segments.next()?.parse::<usize>().ok()?)?)?;
        let direction: TwistDirectionEnum = direction.into();
        let direction = match segments.next()?.parse::<i8>().ok()? {
            1 => direction,
            2 => direction.double()?,
            -1 => direction.rev(),
            -2 => direction.rev().double()?,
            _ => return None,
        };

        let layer = LayerEnum::from_primitive(segments.next()?.parse().ok()?);
        if segments.next().is_some() {
            return None;
        }
        Some(Twist::new(face, direction, layer))
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
        let sticker_id = MC4D_TWIST_IDS[&(self.face, self.direction)];
        let direction_id = 1;
        let layer_mask = self.layer as u8;
        format!("{sticker_id},{direction_id},{layer_mask}")
    }

    fn mc4d_twist_order() -> impl Iterator<Item = Option<(Face, TwistDirectionEnum)>> {
        // Intentionally switch I and O
        const MC4D_FACE_ORDER: [Face; 8] = [
            Face::O,
            Face::B,
            Face::D,
            Face::L,
            Face::R,
            Face::U,
            Face::F,
            Face::I,
        ];

        MC4D_FACE_ORDER.into_iter().flat_map(|face| {
            let mut basis = face.basis_faces();
            basis.sort_by_key(|f| f.axis()); // order: X, Y, Z, W
            basis.reverse(); // order: W, Z, Y, X

            let piece_locations =
                itertools::iproduct!([-1, 0, 1], [-1, 0, 1], [-1, 0, 1]).map(|(x, y, z)| [x, y, z]);
            let corners = piece_locations
                .clone()
                .filter(|v| Vector3::from(*v).magnitude2() == 3);
            let edges = piece_locations
                .clone()
                .filter(|v| Vector3::from(*v).magnitude2() == 2);
            let centers = piece_locations.filter(|v| Vector3::from(*v).magnitude2() == 1);
            let core = std::iter::once([0, 0, 0]);
            let mc4d_order_piece_locations = corners.chain(edges).chain(centers).chain(core);

            mc4d_order_piece_locations
                .map(move |mc4d_coords_of_sticker_within_face: [i8; 3]| {
                    let mut offset = [0; 4];
                    for i in 0..3 {
                        offset[basis[i].axis() as usize] += mc4d_coords_of_sticker_within_face[i];
                    }

                    TwistDirectionEnum::from_signs_within_face(Self::signs_within_face(
                        face,
                        match face {
                            Face::O => offset.into(), // not sure why this is necessary, but it is
                            _ => -Vector4::from(offset),
                        },
                    ))
                })
                .map(move |twist_dir| Some((face.into(), twist_dir?)))
        })
    }

    fn signs_within_face(face: Face, piece_loc_signs: Vector4<i8>) -> Vector3<i8> {
        let [basis1, basis2, basis3] = face.basis();
        cgmath::vec3(
            piece_loc_signs.dot(basis1.cast().unwrap()),
            piece_loc_signs.dot(basis2.cast().unwrap()),
            piece_loc_signs.dot(basis3.cast().unwrap()),
        )
    }

    pub fn iter_all_twists() -> impl Iterator<Item = Twist> {
        iproduct!(Face::iter(), TwistDirectionEnum::iter())
            .map(|(face, direction)| Twist::new(face, direction, LayerEnum::This))
    }
}

// From Hyperspeedcube
// https://github.com/HactarCE/Hyperspeedcube/blob/645bbd3e88eec62d25a22c835a7174a0b2f44f99/src/piecepuzzle/common.rs
#[derive(FromPrimitive, Debug, Default, Copy, Clone, PartialEq, Eq, Hash, EnumIter)]
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
    fn symbol_on_face(self, face: Face) -> String {
        if face == Face::O {
            return self.rev().symbol_on_face(Face::I);
        }

        let vector4 = (face.basis_matrix() * self.vector3_f32().extend(0.0))
            .cast::<i8>()
            .unwrap();

        fn select_face_char(x: i8, char_pos: &'static str, char_neg: &'static str) -> &'static str {
            match x {
                1 => char_pos,
                -1 => char_neg,
                _ => "",
            }
        }

        // "UFRO" is the most natural-sounding order IMO.
        String::new()
            + select_face_char(vector4.y, "U", "D")
            + select_face_char(vector4.z, "F", "B")
            + select_face_char(vector4.x, "R", "L")
            + select_face_char(vector4.w, "O", "I")
            + if self.is_face_180() { "2" } else { "" }
    }

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

    fn period(self) -> usize {
        use TwistDirectionEnum::*;

        match self {
            // 90-degree face (2c) twists.
            R | L | U | D | F | B => 4,
            // 180-degree face (2c) twists.
            R2 | L2 | U2 | D2 | F2 | B2 => 2,
            // 180-degree edge (3c) twists.
            UF | DB | UR | DL | FR | BL | DF | UB | UL | DR | BR | FL => 2,
            // 120-degree corner (4c) twists.
            UFR | DBL | UFL | DBR | DFR | UBL | UBR | DFL => 3,
        }
    }
    fn rev(self) -> Self {
        Self::from(self as u8 ^ 1)
    }

    fn mirror(self, axis: Axis) -> Self {
        if axis == Axis::W {
            return self;
        }
        let mut v = self.vector3();
        v *= -1;
        v[axis as usize] *= -1;
        let ret = Self::from_signs_within_face(v).unwrap();
        if self.is_face_180() {
            ret.double().unwrap()
        } else {
            ret
        }
    }

    fn half(self) -> Option<Self> {
        use TwistDirectionEnum::*;

        match self {
            R2 | L2 | U2 | D2 | F2 | B2 => Some(Self::from(self as u8 - 6)),
            _ => None,
        }
    }
    fn is_face_180(self) -> bool {
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

    fn vector3(self) -> Vector3<i8> {
        use TwistDirectionEnum::*;

        let x = match self {
            R | R2 | UR | FR | DR | BR | UFR | DBR | DFR | UBR => 1, // R
            L | L2 | UL | FL | DL | BL | UFL | DBL | DFL | UBL => -1, // L
            U | D | F | B | U2 | D2 | F2 | B2 | UF | DB | DF | UB => 0,
        };
        let y = match self {
            U | U2 | UF | UR | UB | UL | UFR | UFL | UBL | UBR => 1, // U
            D | D2 | DF | DR | DB | DL | DFR | DFL | DBL | DBR => -1, // D
            R | L | F | B | R2 | L2 | F2 | B2 | FR | BL | BR | FL => 0,
        };
        let z = match self {
            F | F2 | UF | FR | DF | FL | UFR | UFL | DFR | DFL => 1, // F
            B | B2 | UB | BR | DB | BL | UBR | UBL | DBR | DBL => -1, // B
            R | L | U | D | R2 | L2 | U2 | D2 | UR | DL | UL | DR => 0,
        };

        vec3(x, y, z)
    }
    fn vector3_f32(self) -> Vector3<f32> {
        self.vector3().cast().unwrap()
    }
    fn from_signs_within_face(v: Vector3<i8>) -> Option<Self> {
        use TwistDirectionEnum::*;

        match [v.x, v.y, v.z] {
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
