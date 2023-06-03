//! Cube representation based on a permutation of all pieces and an orientation
//! for each piece.

use super::*;

use crate::{groups::A4, piece_cube::puzzle::PieceCube};

/// A cube representation for computing moves quickly as long as they don't affect the LDBO piece
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CubieCube {
    pub orientation: Orientation<A4>,
    pub permutation: Permutation,
}

impl From<PieceCube> for CubieCube {
    fn from(cube: PieceCube) -> Self {
        CubieCube {
            orientation: cube.into(),
            permutation: cube.into(),
        }
    }
}

impl CubieCube {
    /// Returns the solved state
    pub fn solved() -> CubieCube {
        CubieCube::default()
    }

    /// Applies the given move to the cubiecube
    pub fn apply_move(self, m: Move) -> CubieCube {
        CubieCube {
            orientation: self
                .orientation
                .permute(PERM_MOVE_TABLE[m.0 as usize])
                .apply_orientation(A4_MOVE_TABLE[m.0 as usize]),
            permutation: self.permutation.permute(PERM_MOVE_TABLE[m.0 as usize]),
        }
    }

    /// Applies the moves to the cubiecube
    pub fn apply_moves(self, moves: impl IntoIterator<Item = Move>) -> CubieCube {
        let mut result = self;
        for m in moves {
            result = result.apply_move(m);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::{node_cube::Node, phases::Phase};

    use super::*;

    #[test]
    fn test_cubie_cube_twists() {
        for i in 0..HYPERSOLVE_TWISTS.len() {
            let cubiecube = CubieCube::from(PieceCube::solved()).apply_move(Move(i as u8));
            let piececube = PieceCube::solved().twist(HYPERSOLVE_TWISTS[i]);

            assert!(cubiecube == CubieCube::from(piececube))
        }
    }

    #[test]
    fn test_phase2_pruning_table_with_cubie_cube() {
        use crate::node_cube::Phase2Node;
        use crate::phases::Phase2;
        use itertools::Itertools;
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Phase2Node::goal().get_index());

        let mut one_move_states = Vec::new();

        (0..Phase2::N_MOVES).for_each(|move_index| {
            let cube = CubieCube::solved().apply_move(Move(move_index as u8));
            let phase2_node = Phase2Node::from(cube);
            one_move_states.push(cube);
            set.insert(phase2_node.get_index());
        });

        one_move_states
            .into_iter()
            .cartesian_product(0..Phase2::N_MOVES)
            .for_each(|(cube, move_index)| {
                let cube = cube.apply_move(Move(move_index as u8));
                let phase2_node = Phase2Node::from(cube);
                set.insert(phase2_node.get_index());
            });

        // Should have found 152 nodes
        assert_eq!(set.len(), 152)
    }
}
