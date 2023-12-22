use super::*;
use crate::{
    node_cube::{Node, Phase1Node, Phase2Node, Phase3Node},
    prune::{ArrayPruningTable, HashMapPruningTable, PruningTable},
};

/// Upper bound on 2^4 God's number (STM)
///
/// A solution will always be found in this number of moves or less.
pub const GODS_NUMBER_UPPER_BOUND: u32 =
    (Phase1::MAX_DEPTH + Phase2::MAX_DEPTH + Phase3::MAX_DEPTH) as u32;

pub trait Phase {
    /// Number of allowed moves in this phase
    const N_MOVES: usize;
    /// God's number for this phase
    const MAX_DEPTH: u8;
    /// Phase index (1, 2, or 3)
    const PHASE_INDEX: usize;
    /// Node type used in this phase
    type Node: Node;
    /// Pruning table type used in this phase
    type PruningTable: PruningTable<Self::Node>;

    /// Pruning table depth for this phase
    const PRUNING_DEPTH: u8;
    /// An iterator over all moves in this phase
    const MOVE_ITERATOR: cubie_cube::MoveIterator =
        cubie_cube::MoveIterator::new(cubie_cube::Move(0)..cubie_cube::Move(Self::N_MOVES as u8));
}

pub struct Phase1 {}

impl Phase for Phase1 {
    const N_MOVES: usize = 92;
    const MAX_DEPTH: u8 = 8;
    const PRUNING_DEPTH: u8 = 5;
    const PHASE_INDEX: usize = 0;
    type Node = Phase1Node;
    type PruningTable = HashMapPruningTable<Phase1Node>;
}

pub struct Phase2 {}

impl Phase for Phase2 {
    const N_MOVES: usize = 44;
    const MAX_DEPTH: u8 = 10;
    const PRUNING_DEPTH: u8 = 6;
    const PHASE_INDEX: usize = 1;
    type Node = Phase2Node;
    type PruningTable = HashMapPruningTable<Phase2Node>;
}

pub struct Phase3 {}

impl Phase for Phase3 {
    const N_MOVES: usize = 12;
    const MAX_DEPTH: u8 = 21;
    const PRUNING_DEPTH: u8 = Self::MAX_DEPTH;
    const PHASE_INDEX: usize = 2;
    type Node = Phase3Node;
    type PruningTable = ArrayPruningTable<Phase3Node>;
}
