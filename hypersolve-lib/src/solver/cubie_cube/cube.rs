use crate::puzzle::Cube;
use crate::solver::node_cube::*;

use super::*;

/// A cube representation for computing moves quickly as long as they don't affect the LDBO piece
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(crate) struct CubieCube {
    pub(crate) orientation: Orientation<crate::common::groups::A4>,
    pub(crate) permutation: Permutation,
}

impl From<Cube> for CubeIndex {
    fn from(value: Cube) -> Self {
        CubieCube::from(value).into()
    }
}

impl From<CubieCube> for CubeIndex {
    fn from(value: CubieCube) -> Self {
        let phase1_index = Phase1Node::from(value).get_index() as u128;
        let phase2_index = Phase2Node::from(value).get_index() as u128;
        let phase3_index = Phase3Node::from(value).get_index() as u128;

        CubeIndex(
            phase1_index * Phase2Node::N_STATES as u128 * Phase3Node::N_STATES as u128
                + phase2_index * Phase3Node::N_STATES as u128
                + phase3_index,
        )
    }
}

impl From<CubeIndex> for CubieCube {
    fn from(value: CubeIndex) -> Self {
        let mut index = value.0;

        let phase3_node =
            Phase3Node::from_index((index % Phase3Node::N_STATES as u128) as u64, None);
        index /= Phase3Node::N_STATES as u128;

        let phase2_node =
            Phase2Node::from_index((index % Phase2Node::N_STATES as u128) as u64, None);
        index /= Phase2Node::N_STATES as u128;

        let phase1_node = Phase1Node::from_index(index as u64, None);

        let permutation = Permutation::from_coords(
            phase2_node.io_coord,
            phase3_node.i_coord,
            phase3_node.o_coord,
        );

        let orientation =
            Orientation::from_k4_c3_coords(phase1_node.get_index(), phase2_node.c3_coord);

        CubieCube {
            orientation,
            permutation,
        }
    }
}

impl From<Cube> for CubieCube {
    fn from(cube: Cube) -> Self {
        CubieCube {
            orientation: cube.into(),
            permutation: cube.into(),
        }
    }
}

impl CubieCube {
    /// Applies the given move to the cubiecube
    pub(crate) fn apply_move(self, m: Move) -> CubieCube {
        CubieCube {
            orientation: self
                .orientation
                .permute(PERM_MOVE_TABLE[m.0 as usize])
                .apply_orientation(A4_MOVE_TABLE[m.0 as usize]),
            permutation: self.permutation.permute(PERM_MOVE_TABLE[m.0 as usize]),
        }
    }

    /// Applies the moves to the cubiecube
    pub(crate) fn apply_moves<'a>(self, moves: impl Iterator<Item = &'a Move>) -> CubieCube {
        let mut result = self;
        for m in moves {
            result = result.apply_move(*m);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::puzzle::Cube;
    use crate::solver::{node_cube::Node, phases::Phase};

    use super::*;

    #[test]
    fn test_cubie_cube_twists() {
        for (i, &twist) in HYPERSOLVE_TWISTS.iter().enumerate() {
            let cubiecube = CubieCube::from(Cube::solved()).apply_move(Move(i as u8));
            let piececube = Cube::solved().twist(twist);

            assert!(cubiecube == CubieCube::from(piececube))
        }
    }

    #[test]
    fn test_phase2_pruning_table_with_cubie_cube() {
        use crate::solver::node_cube::Phase2Node;
        use crate::solver::phases::Phase2;
        use itertools::Itertools;
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Phase2Node::goal().get_index());

        let mut one_move_states = Vec::new();

        (0..Phase2::N_MOVES).for_each(|move_index| {
            let cube = CubieCube::default().apply_move(Move(move_index as u8));
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

    #[test]
    fn to_from_index() {
        for i in 0..100 {
            let index = CubeIndex::try_from(i * 33500489927290203486927204 + 17).unwrap();

            assert_eq!(index, CubieCube::from(index).into());
        }
    }
}
