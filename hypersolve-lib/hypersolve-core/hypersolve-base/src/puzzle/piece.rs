use std::fmt::{Debug, Display};

use const_for::const_for;

use crate::*;

/// A piece on the cube represented by a vector of 4 faces. Each face represents the
/// face on which the sticker from that axis currently is.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Piece {
    /// The face that the piece on the given axis is currently on
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
    pub const fn new(faces: [Face; 4]) -> Piece {
        Piece { faces }
    }

    pub const fn to_a4(self) -> A4 {
        // get the permutation of the axes of the piece
        let mut axis_permutation = self.to_axis_permutation();

        // if the piece is at an odd location then swap the stickers on the X and Y axis
        // becase moves that shouldn't affect orientation swap the stickers on the X and Y axis
        if self.current_location().parity().is_odd() {
            axis_permutation = axis_permutation.const_swap(0, 1);
        };

        // if we need to fix the parity of the piece to make it an A4 element then swap which
        // stickers are the X and Y stickers because we don't distinguish them for orientation purposes
        if axis_permutation.parity().is_odd() {
            axis_permutation = axis_permutation.inverse().const_swap(0, 1).inverse();
        }

        // convert the permutation to an A4 element
        // SAFTEY: We just ensured that the axis permutation is an even permutation so it is a valid A4 element
        unsafe { A4::from_permutation_unchecked(axis_permutation) }
    }

    pub const fn const_eq(&self, other: &Self) -> bool {
        const_for!(i in 0..self.faces.len() => {
            if self.faces[i] as u8 != other.faces[i] as u8 {
                return false;
            }
        });
        true
    }

    /// Returns the axis permutation of the piece in the "is replaced by format"
    pub const fn to_axis_permutation(self) -> GenericPermutation<4> {
        // SAFTEY: A piece contains a sticker on every axis so this is a valid permutation
        unsafe {
            GenericPermutation::from_array_unchecked(
                const_arr!([u8; 4], |i| self.faces[i].axis() as u8)
            )
            .inverse()
        }
    }

    pub const fn current_location(&self) -> PieceLocation {
        PieceLocation::from_piece(self)
    }

    pub const fn is_affected_by_twist(self, twist: &Twist) -> bool {
        let twist_face = match twist.layer {
            Layer::This => twist.face,
            Layer::Other => twist.face.opposite(),
            Layer::Both => return true,
        };

        const_for!(i in 0..self.faces.len() => {
            if self.faces[i] as u8 == twist_face as u8 {
                return true;
            }
        });

        false
    }

    const fn rotate(mut self, from: Axis, to: Axis) -> Self {
        const_for!(i in 0..self.faces.len() => {
            if self.faces[i].axis() as u8 == from as u8 {
                self.faces[i] = Face::from_axis_sign(to, self.faces[i].sign())
            } else if self.faces[i].axis() as u8 == to as u8 {
                self.faces[i] = Face::from_axis_sign(from, self.faces[i].sign())
            }
        });

        self.mirror(from) // Flip sign of one axis
    }

    const fn rotate_by_faces(self, from: Face, to: Face) -> Self {
        if from.sign() as u8 == to.sign() as u8 {
            self.rotate(from.axis(), to.axis())
        } else {
            self.rotate(to.axis(), from.axis())
        }
    }

    const fn mirror(mut self, axis: Axis) -> Self {
        const_for!(i in 0..self.faces.len() => {
            if self.faces[i].axis() as u8 == axis as u8 {
                self.faces[i] = self.faces[i].opposite();
            }
        });

        self
    }

    pub const fn twist(mut self, twist: Twist) -> Self {
        let [basis_x, basis_y, basis_z] = twist.face.basis_faces();

        let dirs = match twist.face {
            // The O face is backwards so we have to twist in the opposite direction
            // This is analogous to how the F face of a 3D cube is backwards in this
            // projection because we are looking at it from the other side
            //       _______________
            //       |\           /|
            //       | \    U    / |
            //       |  +-------+  |
            //       |  |   B   |  |
            //       |L |       | R|
            //       |  +-------+  |
            //       | /    D    \ |
            //       |/___________\|
            Face::O => twist.direction.reverse(),
            _ => twist.direction,
        }
        .dirs_3d();

        const_for!(i in 0..dirs.len() => {
            let (axis, sign) = dirs[i];

            let [a, b] = match axis {
                Axis::X => [basis_z, basis_y],
                Axis::Y => [basis_x, basis_z],
                Axis::Z => [basis_y, basis_x],
                Axis::W => unsafe{std::hint::unreachable_unchecked()},
            };

            if sign as u8 == Sign::Pos as u8 {
                self = self.rotate_by_faces(a, b);
            } else {
                self = self.rotate_by_faces(b, a);
            }

        });

        self
    }
}

/// Describes locations that pieces can be in
#[derive(Default, Copy, Clone, PartialEq)]
pub struct PieceLocation(pub [Sign; 4]);

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
    /// All piece locations in order of index
    pub const ALL: [Self; 16] = const_arr!([Self; 16], |i| {
        // SAFTEY: i ranges from 0..16
        unsafe { Self::from_index(PieceLocationIndex::from_u8_unchecked(i as u8)) }
    });

    pub const LAST: Self = Self::ALL[15];

    pub const fn const_eq(&self, other: &PieceLocation) -> bool {
        const_for!(i in 0..self.0.len() => {
            if self.0[i] as u8 != other.0[i] as u8 {
                return false
            }
        });
        true
    }

    pub const fn from_piece(piece: &Piece) -> Self {
        let mut faces = piece.faces;

        const_for!(i in 0..faces.len() => {
            faces[piece.faces[i].axis() as u8 as usize] = piece.faces[i];
        });

        PieceLocation::from_signs(
            faces[0].sign(),
            faces[1].sign(),
            faces[2].sign(),
            faces[3].sign(),
        )
    }

    /// Returns the piece that is solved in this location
    pub const fn solved_piece(&self) -> Piece {
        Piece::new([
            Face::from_axis_sign(Axis::X, self.0[0]),
            Face::from_axis_sign(Axis::Y, self.0[1]),
            Face::from_axis_sign(Axis::Z, self.0[2]),
            Face::from_axis_sign(Axis::W, self.0[3]),
        ])
    }

    /// Returns a piece location from signs along each axis
    pub const fn from_signs(x: Sign, y: Sign, z: Sign, w: Sign) -> PieceLocation {
        PieceLocation([x, y, z, w])
    }

    pub const fn index(&self) -> PieceLocationIndex {
        // Toggle the w bit to make face I get indexed before O
        PieceLocationIndex(
            (self.0[0].to_binary() * 2_usize.pow(0)
                + self.0[1].to_binary() * 2_usize.pow(1)
                + self.0[2].to_binary() * 2_usize.pow(2)
                + (self.0[3].to_binary() ^ 1) * 2_usize.pow(3)) as u8,
        )
    }

    pub const fn from_index(index: PieceLocationIndex) -> Self {
        // Toggle the w bit to make face I get indexed before O
        let w = (index.0 >> 3 & 0b00000001) ^ 1;
        let z = index.0 >> 2 & 0b00000001;
        let y = index.0 >> 1 & 0b00000001;
        let x = index.0 & 0b00000001;

        // SAFTEY: w, z, y, and x are created from single bits so they are either 0 or 1
        unsafe {
            PieceLocation::from_signs(
                Sign::from_binary_unchecked(x as usize),
                Sign::from_binary_unchecked(y as usize),
                Sign::from_binary_unchecked(z as usize),
                Sign::from_binary_unchecked(w as usize),
            )
        }
    }

    /// Gets the parity of this piece location
    pub const fn parity(&self) -> Parity {
        match Sign::product(self.0) {
            Sign::Pos => Parity::Even,
            Sign::Neg => Parity::Odd,
        }
    }

    /// Returns whether this piece location is affected by the twist
    pub const fn is_affected_by_twist(self, twist: &Twist) -> bool {
        let twist_sign = match twist.layer {
            Layer::Both => return true,
            Layer::This => twist.face.sign(),
            Layer::Other => twist.face.sign().other(),
        };

        let axis = twist.face.axis();
        let loc_sign = self.0[axis as usize];

        twist_sign as u8 == loc_sign as u8
    }
}

/// An index for a piece location
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PieceLocationIndex(u8);

impl PieceLocationIndex {
    /// The last piece location index
    pub const LAST: Self = Self(15);

    /// Creates a new piece location index from a u8
    ///
    /// # Safety
    /// index must be less than 16
    pub const unsafe fn from_u8_unchecked(index: u8) -> Self {
        debug_assert!(index < 16);
        PieceLocationIndex(index)
    }

    /// Creates a new piece location index from a u8
    pub const fn from_u8(index: u8) -> Option<Self> {
        if index < 16 {
            // SAFTEY: we just verified that this is valid
            unsafe { Some(PieceLocationIndex::from_u8_unchecked(index)) }
        } else {
            None
        }
    }

    /// Returns the index as a u8
    pub const fn into_u8(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_location_index() {
        for i in 0..16 {
            assert_eq!(
                PieceLocation::from_index(PieceLocationIndex(i)).index(),
                PieceLocationIndex(i)
            )
        }
    }

    #[test]
    fn test_piece_current_location() {
        for i in 0..16 {
            assert_eq!(
                PieceLocation::from_index(PieceLocationIndex(i))
                    .solved_piece()
                    .current_location(),
                PieceLocation::from_index(PieceLocationIndex(i))
            )
        }

        assert_ne!(
            PieceLocation::from_index(PieceLocationIndex(6))
                .solved_piece()
                .current_location(),
            PieceLocation::from_index(PieceLocationIndex(1))
        )
    }
}
