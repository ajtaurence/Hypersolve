use hypersolve_base::{Cube, Orientation, Permutation, A4};

use crate::*;

/// A cube representation for computing moves quickly as long as they don't affect the LDBO piece
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CubieCube {
    pub orientation: Orientation<A4>,
    pub permutation: Permutation,
}

impl CubieCube {
    pub fn from_cube(cube: Cube) -> Self {
        let cube = cube.reposition();
        CubieCube {
            orientation: Orientation::<A4>::from_cube(cube),
            permutation: Permutation::from_cube(cube),
        }
    }

    /// Applies the move to the cubiecube
    pub fn apply_move<P: Phase>(self, m: Move<P>) -> CubieCube {
        CubieCube {
            orientation: self
                .orientation
                .permute(m.permutation())
                .apply_orientation(m.orientation()),
            permutation: self.permutation.permute(m.permutation()),
        }
    }

    /// Applies the moves to the cubiecube
    pub fn apply_moves<P: Phase>(self, moves: impl MoveIterator<P>) -> CubieCube {
        moves.fold(self, |cube, m| cube.apply_move(m))
    }
}
