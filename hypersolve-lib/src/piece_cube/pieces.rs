use super::*;

use crate::{common::*, groups::Permutation};

/// A piece on the cube represented by a vector of 4 faces. Each face represents the
/// face on which the sticker from that axis currently is.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Piece {
    /// The face that the piece on the given axis is currently on
    pub faces: Vector4<Face>,
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            self.faces[0], self.faces[1], self.faces[2], self.faces[3]
        )
    }
}

impl std::fmt::Debug for Piece {
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

impl std::ops::Index<Axis> for Piece {
    type Output = Face;

    fn index(&self, index: Axis) -> &Self::Output {
        &self.faces[index as usize]
    }
}

impl std::ops::IndexMut<Axis> for Piece {
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        &mut self.faces[index as usize]
    }
}

impl Piece {
    pub const fn new(faces: Vector4<Face>) -> Piece {
        Piece { faces }
    }

    /// Returns the axis permutation of the piece in the "is replaced by format"
    pub fn to_axis_permutation(self) -> Permutation<4> {
        Permutation::from_array(self.faces.map(|f| f.axis() as usize).0).inverse()
    }

    pub fn current_location(self) -> PieceLocation {
        let mut solved_faces = self.faces;
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
        for face in &mut self.faces.0 {
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
        for face in &mut self.faces.0 {
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

/// Describes locations that pieces can be in rather than pieces themselves
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct PieceLocation(pub Vector4<Sign>);

impl From<Piece> for PieceLocation {
    fn from(piece: Piece) -> Self {
        let mut faces = piece.faces;
        faces.sort();
        PieceLocation::from_signs(
            faces[0].sign(),
            faces[1].sign(),
            faces[2].sign(),
            faces[3].sign(),
        )
    }
}

impl std::fmt::Display for PieceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.solved_piece())
    }
}

impl std::fmt::Debug for PieceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.solved_piece())
    }
}

impl std::ops::Index<Axis> for PieceLocation {
    type Output = Sign;
    fn index(&self, axis: Axis) -> &Self::Output {
        match axis {
            Axis::X => &self.0[0],
            Axis::Y => &self.0[1],
            Axis::Z => &self.0[2],
            Axis::W => &self.0[3],
        }
    }
}

impl std::ops::IndexMut<Axis> for PieceLocation {
    fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
        match axis {
            Axis::X => &mut self.0[0],
            Axis::Y => &mut self.0[1],
            Axis::Z => &mut self.0[2],
            Axis::W => &mut self.0[3],
        }
    }
}

impl PieceLocation {
    /// Returns the piece that is solved in this location
    pub const fn solved_piece(&self) -> Piece {
        Piece::new(Vector4::from_array([
            Face::from_axis_sign(Axis::X, self.0 .0[0]),
            Face::from_axis_sign(Axis::Y, self.0 .0[1]),
            Face::from_axis_sign(Axis::Z, self.0 .0[2]),
            Face::from_axis_sign(Axis::W, self.0 .0[3]),
        ]))
    }

    /// Returns a piece location from signs along each axis
    pub const fn from_signs(x: Sign, y: Sign, z: Sign, w: Sign) -> PieceLocation {
        PieceLocation(Vector::<_, 4>([x, y, z, w]))
    }

    /// Gets the parity of this piece location
    pub fn parity(&self) -> Parity {
        match self.0[0] * self.0[1] * self.0[2] * self.0[3] {
            Sign::Pos => Parity::Even,
            Sign::Neg => Parity::Odd,
        }
    }

    /// Iterates over all piece locations
    pub fn iter() -> impl Iterator<Item = Self> {
        (0..=16).map(PieceLocation::from_index)
    }

    /// Gets the index of this location
    pub const fn index(self) -> usize {
        // Toggle the w bit to make face I get indexed before O
        self.0 .0[0].to_binary() * 2_usize.pow(0)
            + self.0 .0[1].to_binary() * 2_usize.pow(1)
            + self.0 .0[2].to_binary() * 2_usize.pow(2)
            + (self.0 .0[3].to_binary() ^ 1) * 2_usize.pow(3)
    }

    /// Gets the location from this index
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

    /// Returns whether this piece location is affected by the given twist
    pub fn is_affected_by_twist(self, twist: Twist) -> bool {
        self.solved_piece().is_affected_by_twist(twist)
    }
}
