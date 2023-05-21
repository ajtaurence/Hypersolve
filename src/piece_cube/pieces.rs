use std::{
    fmt::{Debug, Display},
    ops::{Index, IndexMut},
};

use crate::common::{Axis, Face, Sign};
use crate::cubiecube::groups::{Parity, PermutationWord};

use super::*;

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

    /// Returns the axis permutation of the piece in the "is replaced by format"
    pub fn to_axis_permutation(&self) -> PermutationWord<4> {
        PermutationWord(self.faces.map(|f| f.axis() as usize)).inverse()
    }

    pub fn current_location(self) -> PieceLocation {
        let mut solved_faces = self.faces.clone();
        solved_faces.sort();

        Piece::new(solved_faces).into()
    }

    pub fn is_affected_by_twist(self, twist: Twist) -> bool {
        match twist.layer {
            LayerEnum::This => self.faces.contains(&twist.face),
            LayerEnum::Other => self.faces.contains(&twist.face.opposite()),
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

    /// Gets the parity of this piece location
    pub fn parity(&self) -> Parity {
        match self.x * self.y * self.z * self.w {
            Sign::Pos => Parity::Even,
            Sign::Neg => Parity::Odd,
        }
    }

    /// Iterates over all piece locations
    pub fn iter() -> impl Iterator<Item = Self> {
        (0..=16).map(|i| PieceLocation::from_index(i))
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
