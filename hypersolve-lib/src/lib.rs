//! Hypersolve-lib is a library implementing a 3-phase solver for the 4D [2<sup>4</sup> Rubik's cube](https://hypercubing.xyz/puzzles/2x2x2x2).
//! The solver guarantees a solution of length of at most 39 moves ([STM](https://hypercubing.xyz/notation/#turn-metrics)), thereby lowering
//! the upper bound on God's number for the 2<sup>4</sup> to 39.

#[macro_use]
mod common;
pub mod puzzle;
pub mod solver;
