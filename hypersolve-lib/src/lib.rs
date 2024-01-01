//! Hypersolve-lib is a library implementing a 3-phase solver for the 4D [2<sup>4</sup> Rubik's cube](https://hypercubing.xyz/puzzles/2x2x2x2).
//! The solver guarantees a solution of length of at most 39 moves ([STM](https://hypercubing.xyz/notation/#turn-metrics)), thereby lowering
//! the upper bound on God's number for the 2<sup>4</sup> to 39.

#[macro_use]
mod macros;
#[macro_use]
mod common;
mod cubie_cube;
mod groups;
mod math;
mod node_cube;
mod phases;
pub(crate) mod piece_cube;
mod prune;
mod solve;

pub use common::{Axis, Face, Sign};
pub use cubie_cube::N_CUBE_STATES;
pub use phases::GODS_NUMBER_UPPER_BOUND;
pub use piece_cube::puzzle::PieceCube as Cube;
pub use piece_cube::{
    CubeIndex, CubeIndexError, LayerEnum, Notation, Twist, TwistDirectionEnum, TwistParseError,
    TwistSequence,
};
pub use solve::{fast_solve, find_scramble};
