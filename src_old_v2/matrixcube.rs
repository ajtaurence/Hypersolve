//! Cube representation based on a matrix for each piece.

use crate::common::Matrix;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MatrixCube {
    pieces: [Matrix; 16],
}

impl MatrixCube {
    pub fn solved() -> Self {
        Self::default()
    }
}
