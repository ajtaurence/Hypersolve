use crate::{
    cubie_cube::{Move, MoveIterator},
    node_cube::{Node, Phase1Node, Phase2Node, Phase3Node},
    prune::{ArrayPruningTable, HashMapPruningTable, PruningTable},
};

pub trait Phase {
    const N_MOVES: usize;
    const MAX_DEPTH: u8;
    const PHASE_INDEX: usize;
    type Node: Node;
    type PruningTable: PruningTable<Self::Node>;

    const PRUNING_DEPTH: u8;
    const MOVE_ITERATOR: MoveIterator = MoveIterator::new(Move(0)..Move(Self::N_MOVES as u8));
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
