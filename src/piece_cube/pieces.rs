use std::{
    fmt::{Debug, Display},
    ops::{Index, IndexMut, Mul, Neg},
};

use cgmath::{Matrix4, Vector4, Zero};
use num_enum::FromPrimitive;
use strum::EnumIter;

use super::*;

#[derive(Copy, Clone, PartialEq, Eq, Default, Hash)]
#[repr(i8)]
pub enum Sign {
    #[default]
    Pos = 1,
    Neg = -1,
}

impl Debug for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sign::Pos => write!(f, "Sign: +"),
            Sign::Neg => write!(f, "Sign: -"),
        }
    }
}

macro_rules! impl_from_int {
    ($type:ty) => {
        impl From<$type> for Sign {
            fn from(value: $type) -> Self {
                match value {
                    1 => Sign::Pos,
                    -1 => Sign::Neg,
                    _ => panic!("cannot convert {:?} to a sign", value),
                }
            }
        }
    };
}

impl_from_int!(i32);
impl_from_int!(i8);

impl Neg for Sign {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Sign::Pos => Sign::Neg,
            Sign::Neg => Sign::Pos,
        }
    }
}

impl Mul for Sign {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match rhs {
            Sign::Pos => self,
            Sign::Neg => -self,
        }
    }
}

impl Sign {
    /// Pos -> 0, Neg -> 1
    const fn to_binary(self) -> usize {
        match self {
            Self::Pos => 0,
            Self::Neg => 1,
        }
    }

    /// 0 -> Pos, 1 -> Neg
    const fn from_binary(value: usize) -> Self {
        match value {
            0 => Self::Pos,
            1 => Self::Neg,
            _ => panic!("cannot convert binary to sign"),
        }
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, FromPrimitive, EnumIter, Default, PartialOrd, Ord, Hash,
)]
#[repr(u8)]
pub enum Axis {
    #[default]
    X = 0,
    Y = 1,
    Z = 2,
    W = 3,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter)]
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

impl Debug for Face {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol_upper_str())
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

impl From<Vector4<i8>> for Face {
    fn from(vec: Vector4<i8>) -> Self {
        match vec {
            Vector4 {
                x,
                y: 0,
                z: 0,
                w: 0,
            } => Face::from_axis_sign(Axis::X, x.into()),
            Vector4 {
                x: 0,
                y,
                z: 0,
                w: 0,
            } => Face::from_axis_sign(Axis::Y, y.into()),
            Vector4 {
                x: 0,
                y: 0,
                z,
                w: 0,
            } => Face::from_axis_sign(Axis::Z, z.into()),
            Vector4 {
                x: 0,
                y: 0,
                z: 0,
                w,
            } => Face::from_axis_sign(Axis::W, w.into()),
            _ => panic!("cannot convert vector {:?} to face", vec),
        }
    }
}

impl Mul<Sign> for Face {
    type Output = Self;

    fn mul(self, rhs: Sign) -> Self::Output {
        match rhs {
            Sign::Pos => self,
            Sign::Neg => self.opposite(),
        }
    }
}

impl Neg for Face {
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

    const fn from_axis_sign(axis: Axis, sign: Sign) -> Self {
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

    const fn opposite(self) -> Face {
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

    const fn symbol_upper_str(self) -> &'static str {
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

    fn vector(self) -> Vector4<f32> {
        (match self.axis() {
            Axis::X => Vector4::unit_x(),
            Axis::Y => Vector4::unit_y(),
            Axis::Z => Vector4::unit_z(),
            Axis::W => Vector4::unit_w(),
        } * self.sign() as i8 as f32)
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

    pub fn basis(self) -> [Vector4<f32>; 3] {
        self.basis_faces().map(|f| f.vector())
    }

    pub fn basis_matrix(self) -> Matrix4<f32> {
        let [x, y, z] = self.basis();
        let w = Vector4::zero();
        Matrix4 { x, y, z, w }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Piece {
    pub faces: [Face; 4],
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            self.faces[0], self.faces[1], self.faces[2], self.faces[3]
        )
    }
}

impl Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}{:?}{:?}{:?}",
            self.faces[0], self.faces[1], self.faces[2], self.faces[3]
        )
    }
}

impl Default for Piece {
    fn default() -> Self {
        PieceLocation::default().solved_piece()
    }
}

impl Index<Axis> for Piece {
    type Output = Face;

    fn index(&self, index: Axis) -> &Self::Output {
        &self.faces[index as usize]
    }
}

impl IndexMut<Axis> for Piece {
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        &mut self.faces[index as usize]
    }
}

impl Piece {
    pub const fn new(faces: [Face; 4]) -> Piece {
        Piece { faces }
    }

    pub fn current_location(self) -> PieceLocation {
        let mut solved_faces = self.faces.clone();
        solved_faces.sort();

        Piece::new(solved_faces).into()
    }

    pub fn is_affected_by_twist(self, twist: Twist) -> bool {
        match twist.layer {
            LayerEnum::This => self.faces.contains(&twist.face),
            LayerEnum::Other => !self.faces.contains(&twist.face),
            LayerEnum::Both => true,
        }
    }
    fn rotate(mut self, from: Axis, to: Axis) -> Self {
        for face in &mut self.faces {
            if face.axis() == from {
                *face = Face::from_axis_sign(to, face.sign())
            } else if face.axis() == to {
                *face = Face::from_axis_sign(from, face.sign())
            }
        }
        self.mirror(from) // Flip sign of one axis
    }

    fn rotate_by_faces(self, from: Face, to: Face) -> Self {
        if from.sign() == to.sign() {
            self.rotate(from.axis(), to.axis())
        } else {
            self.rotate(to.axis(), from.axis())
        }
    }

    fn mirror(mut self, axis: Axis) -> Self {
        for face in &mut self.faces {
            if face.axis() == axis {
                *face = face.opposite();
            }
        }
        self
    }

    pub fn twist(mut self, twist: Twist) -> Self {
        let [basis_x, basis_y, basis_z] = twist.face.basis_faces();

        let mut chars = twist.direction.symbol_xyz().chars().peekable();

        loop {
            let [mut a, mut b] = match chars.next() {
                None => return self,
                Some('x') => [basis_z, basis_y],
                Some('y') => [basis_x, basis_z],
                Some('z') => [basis_y, basis_x],
                _ => unreachable!(),
            };
            let double = chars.next_if_eq(&'2').is_some();
            let inverse = chars.next_if_eq(&'\'').is_some();
            if inverse {
                std::mem::swap(&mut a, &mut b);
            }
            self = self.rotate_by_faces(a, b);
            if double {
                self = self.rotate_by_faces(a, b);
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct PieceLocation {
    x: Sign,
    y: Sign,
    z: Sign,
    w: Sign,
}

impl From<Piece> for PieceLocation {
    fn from(piece: Piece) -> Self {
        let mut faces = piece.faces.clone();
        faces.sort();
        PieceLocation::from_signs(
            faces[0].sign(),
            faces[1].sign(),
            faces[2].sign(),
            faces[3].sign(),
        )
    }
}

impl Display for PieceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.solved_piece())
    }
}

impl Debug for PieceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.solved_piece())
    }
}

impl Index<Axis> for PieceLocation {
    type Output = Sign;
    fn index(&self, axis: Axis) -> &Self::Output {
        match axis {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
            Axis::W => &self.w,
        }
    }
}

impl IndexMut<Axis> for PieceLocation {
    fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
        match axis {
            Axis::X => &mut self.x,
            Axis::Y => &mut self.y,
            Axis::Z => &mut self.z,
            Axis::W => &mut self.w,
        }
    }
}

impl PieceLocation {
    pub const fn solved_piece(&self) -> Piece {
        Piece::new([
            Face::from_axis_sign(Axis::X, self.x),
            Face::from_axis_sign(Axis::Y, self.y),
            Face::from_axis_sign(Axis::Z, self.z),
            Face::from_axis_sign(Axis::W, self.w),
        ])
    }

    pub const fn from_signs(x: Sign, y: Sign, z: Sign, w: Sign) -> PieceLocation {
        PieceLocation { x, y, z, w }
    }

    pub const fn index(self) -> usize {
        // Toggle the w bit to make face I get indexed before O
        self.x.to_binary() * 2_usize.pow(0)
            + self.y.to_binary() * 2_usize.pow(1)
            + self.z.to_binary() * 2_usize.pow(2)
            + (self.w.to_binary() ^ 1) * 2_usize.pow(3)
    }

    pub const fn from_index(index: usize) -> PieceLocation {
        // Toggle the w bit to make face I get indexed before O
        let w = (index >> 3 & 0b00000001) ^ 1;
        let z = index >> 2 & 0b00000001;
        let y = index >> 1 & 0b00000001;
        let x = index & 0b00000001;
        PieceLocation::from_signs(
            Sign::from_binary(x),
            Sign::from_binary(y),
            Sign::from_binary(z),
            Sign::from_binary(w),
        )
    }

    pub fn is_affected_by_twist(self, twist: Twist) -> bool {
        self.solved_piece().is_affected_by_twist(twist)
    }
}
