use super::*;

use crate::{
    common::{Sign, Vector, Vector4},
    groups::Permutation,
};
use itertools::Itertools;
use std::ops::{Index, IndexMut};

/// High level representation of the cube, capable of computing any move but has slower performance
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PieceCube {
    pub pieces: Vector<Piece, 16>,
}

impl Default for PieceCube {
    fn default() -> Self {
        PieceCube::solved()
    }
}

impl Index<PieceLocation> for PieceCube {
    type Output = Piece;
    fn index(&self, index: PieceLocation) -> &Self::Output {
        self.pieces
            .iter()
            .find(|&&piece| PieceLocation::from(piece) == index)
            .unwrap()
    }
}

impl IndexMut<PieceLocation> for PieceCube {
    fn index_mut(&mut self, index: PieceLocation) -> &mut Self::Output {
        self.pieces
            .iter_mut()
            .find(|piece| PieceLocation::from(**piece) == index)
            .unwrap()
    }
}

impl PieceCube {
    const fn new(pieces: Vector<Piece, 16>) -> PieceCube {
        PieceCube { pieces }
    }

    pub fn solved() -> Self {
        PieceCube::new(
            (0..16)
                .map(|i| PieceLocation::from_index(i).solved_piece())
                .collect_vec()
                .try_into()
                .unwrap(),
        )
    }

    pub fn twist(mut self, twist: Twist) -> Self {
        for i in 0..16 {
            if self.pieces[i].is_affected_by_twist(twist) {
                self.pieces[i] = self.pieces[i].twist(twist);
            }
        }
        self
    }

    pub fn twists(mut self, twists: impl IntoIterator<Item = Twist>) -> Self {
        for twist in twists {
            self = self.twist(twist);
        }
        self
    }

    pub fn pieces_except_last(self) -> [Piece; 15] {
        self.pieces
            .into_iter()
            .take(15)
            .collect_vec()
            .try_into()
            .unwrap()
    }

    /// Repositions the inner representation of the cube so the state is the same but the LDBO piece is solved
    pub fn reposition(mut self) -> Self {
        // get the reference sticker
        let (reference_index, &reference_piece) = self
            .pieces
            .iter()
            .find_position(|piece| piece.current_location().index() == 15)
            .unwrap();

        // get the axis permutation of the reference sticker
        let axis_perm = reference_piece.to_axis_permutation();

        // permute the axes of each sticker according to the axis permutation of the reference sticker
        self.pieces = self.pieces.map(|piece| Piece {
            faces: piece.faces.permute(axis_perm),
        });

        // Signs representing the coordinate of the reference piece before
        let reference_signs_before: Vector4<Sign> = PieceLocation::from_index(reference_index)
            .solved_piece()
            .faces
            .cast();

        // Signs representing the coordinate of the reference piece now
        let reference_signs_now: Vector4<Sign> =
            PieceLocation::from_index(15).solved_piece().faces.cast();

        // Get the sign transformation that takes the reference piece from the location it was to the location now
        // Formula for the reference signs now is below:
        // reference_signs_now = reference_signs_before.permute(axis_perm) * transform_signs
        // Solve for transform_signs and we get the following equation
        let transform_signs = reference_signs_before.permute(axis_perm) * reference_signs_now;

        // Now we apply the transformation to every piece and arrive at the permutation taking pieces to their new solved positions
        let piece_perm = Permutation::from_array(
            Permutation::IDENTITY
                .into_inner()
                .map(PieceLocation::from_index)
                .map(|piece_loc| PieceLocation(piece_loc.0.permute(axis_perm) * transform_signs))
                .map(|piece_loc| piece_loc.index()),
        )
        .inverse();

        // Apply this permutation to the pieces to put them in their correct slots
        self.pieces = self.pieces.permute(piece_perm);

        self
    }
}