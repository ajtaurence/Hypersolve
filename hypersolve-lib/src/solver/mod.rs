//! 3-phase solver functionality

mod cube_index;
mod cubie_cube;
mod node_cube;
mod phases;
mod prune;
mod solve;

pub use cube_index::*;
use cubie_cube::*;
use node_cube::*;
use phases::*;
use prune::*;
pub use solve::*;

/// Total number of 2<sup>4</sup> cube states (ignoring cube rotations)
pub const N_CUBE_STATES: u128 =
    Phase1Node::N_STATES as u128 * Phase2Node::N_STATES as u128 * Phase3Node::N_STATES as u128;

/// Upper bound on 2<sup>4</sup> God's number (STM)
///
/// A solution will always be found in this number of moves or less.
pub const GODS_NUMBER_UPPER_BOUND: u32 =
    (Phase1::MAX_DEPTH + Phase2::MAX_DEPTH + Phase3::MAX_DEPTH) as u32;
