use std::ops::{Index, IndexMut};

use itertools::Itertools;

use super::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PieceCube {
    pub pieces: [Piece; 16],
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
    const fn new(pieces: [Piece; 16]) -> PieceCube {
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

    pub fn twist(&mut self, twist: Twist) -> Self {
        for i in 0..16 {
            if self.pieces[i].is_affected_by_twist(twist) {
                self.pieces[i] = self.pieces[i].twist(twist);
            }
        }
        *self
    }

    pub fn pieces_except_last(self) -> [Piece; 15] {
        self.pieces
            .into_iter()
            .take(15)
            .collect_vec()
            .try_into()
            .unwrap()
    }

    /// Repositions the inner representation of the cube so the state is the same but the OBLD piece is solved
    pub fn reposition(self) -> Self {
        //todo!("impliment reposition");
        self
    }
}
