use std::ops::{Index, IndexMut};

use itertools::Itertools;

use crate::{
    common::{Sign, Vector, Vector4},
    groups::Permutation,
};

use super::*;

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

        let reference_signs: Vector4<Sign> =
            PieceLocation::from_index(15).solved_piece().faces.cast();

        let solved_reference_signs: Vector4<Sign> = PieceLocation::from_index(reference_index)
            .solved_piece()
            .faces
            .cast();

        let solved = Self::solved();

        println!(
            "{:?}",
            solved.pieces.map(|piece| Piece { faces: piece.faces })
        );

        // honestly no idea what this is doing ¯\_(ツ)_/¯
        let piece_perm = Permutation::from_array(
            solved
                .pieces
                .map(|piece| Piece {
                    faces: piece.faces * reference_signs * solved_reference_signs,
                })
                .map(|piece| piece.current_location().index())
                .0,
        );

        self.pieces = self.pieces.permute(piece_perm);

        self
    }
}
