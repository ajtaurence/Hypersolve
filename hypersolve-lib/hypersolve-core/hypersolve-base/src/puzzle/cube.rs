use const_for::const_for;

use crate::*;

/// 2<sup>4</sup> cube state
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Debug))]
pub struct Cube {
    pieces: [Piece; 16],
}

impl Default for Cube {
    fn default() -> Self {
        Cube::SOLVED
    }
}

impl std::ops::Index<PieceLocation> for Cube {
    type Output = Piece;
    fn index(&self, index: PieceLocation) -> &Self::Output {
        self.pieces
            .iter()
            .find(|&&piece| PieceLocation::from_piece(&piece) == index)
            .unwrap()
    }
}

impl std::ops::IndexMut<PieceLocation> for Cube {
    fn index_mut(&mut self, index: PieceLocation) -> &mut Self::Output {
        self.pieces
            .iter_mut()
            .find(|piece| PieceLocation::from_piece(piece) == index)
            .unwrap()
    }
}

impl Cube {
    /// The solved cube state
    pub const SOLVED: Self = Self::new(const_arr!([Piece; 16], |i| {
        PieceLocation::ALL[i].solved_piece()
    }));

    const fn new(pieces: [Piece; 16]) -> Cube {
        Cube { pieces }
    }

    /// Returns whether the cube is solved
    pub fn is_solved(&self) -> bool {
        let respositioned = self.reposition();

        const_for!(i in 0..self.pieces.len() => {
            if !respositioned.pieces[i].const_eq(&Self::SOLVED.pieces[i]) {
                return false;
            }
        });
        true
    }

    /// Applies the twist to this cube
    pub const fn twist(mut self, twist: Twist) -> Self {
        const_for!(i in 0..16 => {
            if self.pieces[i].is_affected_by_twist(&twist) {
                self.pieces[i] = self.pieces[i].twist(twist);
            }
        });

        self
    }

    /// Applies a sequence of twists to this cube rotations
    pub const fn twists_const(mut self, twists: &[Twist]) -> Self {
        const_for!(i in 0..twists.len() => {
            self = self.twist(twists[i]);
        });

        self
    }

    /// Applies a sequence of twists to this cube rotations
    pub fn twists(mut self, twists: impl IntoIterator<Item = Twist>) -> Self {
        for twist in twists {
            self = self.twist(twist);
        }
        self
    }

    pub const fn pieces_except_last(&self) -> [Piece; 15] {
        const_arr!([Piece; 15], |i| self.pieces[i])
    }

    /// Repositions the inner representation of the cube so the state is the same but the LDBO piece is solved
    pub const fn reposition(mut self) -> Self {
        const fn find_index_of_last_piece(array: &[Piece]) -> (usize, Piece) {
            const_for!(i in 0..array.len() => {
                if array[i].current_location().const_eq(&PieceLocation::LAST) {
                    return (i, array[i])
                }
            });
            debug_assert!(false);
            unsafe { std::hint::unreachable_unchecked() }
        }

        const fn mul_sign_arrays<const M: usize>(a: &[Sign; M], b: &[Sign; M]) -> [Sign; M] {
            const_arr!([Sign; M], |i| a[i].mul(b[i]))
        }

        // get the reference sticker
        let (reference_index, reference_piece) = find_index_of_last_piece(&self.pieces);

        // get the axis permutation of the reference sticker
        let axis_perm = reference_piece.to_axis_permutation();

        // permute the axes of each sticker according to the axis permutation of the reference sticker
        const_for!(i in 0..self.pieces.len() => {
            self.pieces[i] = Piece::new(axis_perm.permute(&self.pieces[i].faces))
        });

        // Signs representing the coordinate of the reference piece before
        // SAFTEY: reference_index is less than 16 because it is an index of an array with length 16
        let ref_faces_before = PieceLocation::from_index(unsafe {
            PieceLocationIndex::from_u8_unchecked(reference_index as u8)
        })
        .solved_piece()
        .faces;
        let reference_signs_before = const_arr!([Sign; 4], |i| ref_faces_before[i].sign());

        // Signs representing the coordinate of the reference piece now
        const REF_FACES_NOW: [Face; 4] = PieceLocation::LAST.solved_piece().faces;
        const REFERENCE_SIGNS_NOW: [Sign; 4] = const_arr!([Sign; 4], |i| REF_FACES_NOW[i].sign());

        // Get the sign transformation that takes the reference piece from the location it was to the location now
        // Formula for the reference signs now is below:
        // reference_signs_now = reference_signs_before.permute(axis_perm) * transform_signs
        // Solve for transform_signs and we get the following equation
        let transform_signs = mul_sign_arrays(
            &axis_perm.permute(&reference_signs_before),
            &REFERENCE_SIGNS_NOW,
        );

        // Now we apply the transformation to every piece and arrive at the permutation taking pieces to their new solved positions

        let arr = const_arr!([u8; 16], |i| {
            // SAFTEY: i is less than 16 because it is an element of a Permutation<16>
            let piece_loc = unsafe {
                PieceLocation::from_index(PieceLocationIndex::from_u8_unchecked(
                    GenericPermutation::<16>::IDENTITY.as_array()[i],
                ))
            };

            PieceLocation(mul_sign_arrays(
                &axis_perm.permute(&piece_loc.0),
                &transform_signs,
            ))
            .index()
            .into_u8()
        });

        // SAFTEY: idk, trust me bro
        let piece_perm = unsafe { GenericPermutation::from_array_unchecked(arr).inverse() };

        // Apply this permutation to the pieces to put them in their correct slots
        self.pieces = piece_perm.permute(&self.pieces);

        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_solved() {
        for twist in Twist::ALL_TWISTS {
            assert_eq!(
                Cube::SOLVED.twist(twist).is_solved(),
                twist.is_cube_rotation()
            )
        }
    }
}
