use hypersolve_core::{CubieCube, Orientation, Permutation};

use crate::{Node, Phase1Node, Phase2Node, Phase3Node, N_CUBE_STATES};

/// An index representing a specific cube state
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CubeIndex(pub(crate) u128);

impl CubeIndex {
    /// The solved cube index
    pub const SOLVED: Self = CubeIndex(0);
}

impl From<CubieCube> for CubeIndex {
    fn from(value: CubieCube) -> Self {
        let phase1_index = Phase1Node::from(value).index() as u128;
        let phase2_index = Phase2Node::from(value).index() as u128;
        let phase3_index = Phase3Node::from(value).index() as u128;

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

        let phase3_node = Phase3Node::from_index((index % Phase3Node::N_STATES as u128) as u32);
        index /= Phase3Node::N_STATES as u128;

        let phase2_node = Phase2Node::from_index((index % Phase2Node::N_STATES as u128) as u64);
        index /= Phase2Node::N_STATES as u128;

        let phase1_node = Phase1Node::from_index(index as u32);

        let permutation = Permutation::from_coords(
            phase2_node.io_coord,
            phase3_node.i_coord,
            phase3_node.o_coord,
        );

        let orientation =
            unsafe { Orientation::from_k4_c3_coords(phase1_node.index(), phase2_node.c3_coord) };

        CubieCube {
            orientation,
            permutation,
        }
    }
}

/// Errors for converting an integer into a cube index
#[derive(Debug, thiserror::Error)]
pub enum CubeIndexError {
    #[error("index must be less than `3,357,894,533,384,932,272,635,904,000` but it is `{0}`")]
    InvalidIndex(u128),
}

impl TryFrom<u128> for CubeIndex {
    type Error = CubeIndexError;
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        if value < N_CUBE_STATES {
            Ok(CubeIndex(value))
        } else {
            Err(CubeIndexError::InvalidIndex(value))
        }
    }
}
