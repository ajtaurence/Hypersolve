use std::ops::RangeBounds;

use hypersolve_core::{CubieCube, Phase, Phase1, Phase2, Phase3};

use crate::simple_solve::simple_solve;
use crate::{Node, Phase1Node, Phase2Node, Phase3Node};

pub use crate::cube_index::{CubeIndex, CubeIndexError};
pub use crate::fast_solve::FastSolutionIterator;
pub use crate::solution_iterators::{FixedLengthSolutionIterator, ShortestSolutionIterator};
pub use hypersolve_core::{
    Notation, ParseMC4DTwistError, ParseStandardTwistError, ParseTwistError, Twist, TwistSequence,
};

pub use crate::bound::{Bound, BoundIterator};

/// Total number of 2<sup>4</sup> cube states (ignoring cube rotations)
pub const N_CUBE_STATES: u128 =
    Phase1Node::N_STATES as u128 * Phase2Node::N_STATES as u128 * Phase3Node::N_STATES as u128;

/// Upper bound on 2<sup>4</sup> God's number (STM)
///
/// A solution will always be found in this number of moves or less.
pub const GODS_NUMBER_UPPER_BOUND: usize =
    Phase1::MAX_DEPTH + Phase2::MAX_DEPTH + Phase3::MAX_DEPTH;

/// A 2<sup>4</sup> Rubik's Cube
#[derive(Clone, Copy, Default)]
pub struct Cube(pub(crate) hypersolve_core::Cube);

impl Cube {
    /// The solved cube
    pub const SOLVED: Self = Cube(hypersolve_core::Cube::SOLVED);

    /// Returns a new cube with the given twist applied to it
    pub fn twist(&self, twist: Twist) -> Self {
        Self(self.0.twist(twist))
    }

    /// Returns a new cube with the given twist sequence applied to it
    pub fn twist_seq(&self, twist_seq: impl IntoIterator<Item = Twist>) -> Self {
        let mut cube = *self;

        for twist in twist_seq {
            cube = cube.twist(twist);
        }

        cube
    }

    /// Deterministically finds a solution to the cube as quickly as possible
    ///
    /// The solution length is garanteed to be less than or equal to [`GODS_NUMBER_UPPER_BOUND`]
    pub fn fast_solve(&self) -> TwistSequence {
        simple_solve(CubieCube::from_cube(self.0))
            .into_iter()
            .map(|m| *m.twist())
            .collect()
    }

    /// Returns an iterator which non-deterministically returns increasingly shorter solutions
    pub fn fast_solutions(&self, max_solution_len: Option<usize>) -> FastSolutionIterator {
        FastSolutionIterator::new(self.0, max_solution_len)
    }

    /// Returns an iterator over all solutions to this cube in order of increasing length
    pub fn solutions(&self, solution_lengths: impl RangeBounds<usize>) -> ShortestSolutionIterator {
        ShortestSolutionIterator::new(CubieCube::from_cube(self.0), solution_lengths)
    }

    /// Returns an iterator over all solutions to this cube with the given length
    pub fn solutions_with_len(&self, solution_length: usize) -> FixedLengthSolutionIterator {
        FixedLengthSolutionIterator::new(CubieCube::from_cube(self.0), solution_length)
    }

    /// Returns an iterator over bounds on the length of the optimal solution
    pub fn optimal_bounds(&self) -> BoundIterator {
        BoundIterator::new(*self)
    }
}

/// Returns the scramble for the given cube index
pub fn new_scramble(cube_index: CubeIndex) -> TwistSequence {
    let cube = CubieCube::from(cube_index);

    simple_solve(cube).into_iter().map(|m| *m.twist()).collect()
}
