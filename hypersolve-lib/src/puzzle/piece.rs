use super::*;

/// A piece on the cube represented by a vector of 4 faces. Each face represents the
/// face on which the sticker from that axis currently is.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Piece {
    /// The face that the piece on the given axis is currently on
    pub(crate) faces: Vector4<Face>,
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

impl From<Piece> for crate::common::groups::A4 {
    fn from(piece: Piece) -> Self {
        // get the permutation of the axes of the piece
        let mut axis_permutation = piece.to_axis_permutation();

        // if the piece is at an odd location then swap the stickers on the X and Y axis
        // becase moves that shouldn't affect orientation swap the stickers on the X and Y axis
        if piece.current_location().parity().is_odd() {
            axis_permutation = axis_permutation.swap(0, 1);
        };

        // if we need to fix the parity of the piece to make it an A4 element then swap which
        // stickers are the X and Y stickers because we don't distinguish them for orientation purposes
        if axis_permutation.parity().is_odd() {
            axis_permutation = axis_permutation.inverse().swap(0, 1).inverse();
        }

        // convert the permutation to an A4 element
        groups::A4::try_from(axis_permutation).unwrap()
    }
}

impl Piece {
    pub(crate) const fn new(faces: Vector4<Face>) -> Piece {
        Piece { faces }
    }

    /// Returns the axis permutation of the piece in the "is replaced by format"
    pub(crate) fn to_axis_permutation(self) -> groups::Permutation<4> {
        groups::Permutation::from_array(self.faces.map(|f| f.axis() as usize).0).inverse()
    }

    pub(crate) fn current_location(self) -> PieceLocation {
        let mut solved_faces = self.faces;
        solved_faces.sort();

        Piece::new(solved_faces).into()
    }

    pub(crate) fn is_affected_by_twist(self, twist: Twist) -> bool {
        match twist.layer {
            Layer::This => self.faces.contains(&twist.face),
            Layer::Other => self.faces.contains(&twist.face.opposite()),
            Layer::Both => true,
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

    pub(crate) fn twist(mut self, twist: Twist) -> Self {
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
pub(crate) struct PieceLocation(pub Vector4<Sign>);

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
    pub(crate) const fn solved_piece(&self) -> Piece {
        Piece::new(Vector4::from_array([
            Face::from_axis_sign(Axis::X, self.0 .0[0]),
            Face::from_axis_sign(Axis::Y, self.0 .0[1]),
            Face::from_axis_sign(Axis::Z, self.0 .0[2]),
            Face::from_axis_sign(Axis::W, self.0 .0[3]),
        ]))
    }

    /// Returns a piece location from signs along each axis
    pub(crate) const fn from_signs(x: Sign, y: Sign, z: Sign, w: Sign) -> PieceLocation {
        PieceLocation(Vector::<_, 4>([x, y, z, w]))
    }

    /// Gets the parity of this piece location
    pub(crate) fn parity(&self) -> Parity {
        match self.0[0] * self.0[1] * self.0[2] * self.0[3] {
            Sign::Pos => Parity::Even,
            Sign::Neg => Parity::Odd,
        }
    }

    /// Gets the index of this location
    pub(crate) const fn index(self) -> usize {
        // Toggle the w bit to make face I get indexed before O
        self.0 .0[0].to_binary() * 2_usize.pow(0)
            + self.0 .0[1].to_binary() * 2_usize.pow(1)
            + self.0 .0[2].to_binary() * 2_usize.pow(2)
            + (self.0 .0[3].to_binary() ^ 1) * 2_usize.pow(3)
    }

    /// Gets the location from this index
    pub(crate) const fn from_index(index: usize) -> PieceLocation {
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
    #[cfg(feature = "gen-const-data")]
    pub(crate) fn is_affected_by_twist(self, twist: Twist) -> bool {
        self.solved_piece().is_affected_by_twist(twist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_location_index() {
        for i in 0..16 {
            assert_eq!(PieceLocation::from_index(i).index(), i)
        }
    }

    #[test]
    fn test_piece_current_location() {
        for i in 0..16 {
            assert_eq!(
                PieceLocation::from_index(i)
                    .solved_piece()
                    .current_location(),
                PieceLocation::from_index(i)
            )
        }

        assert_ne!(
            PieceLocation::from_index(6)
                .solved_piece()
                .current_location(),
            PieceLocation::from_index(1)
        )
    }
}
