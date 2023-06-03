use crate::{
    cubie_cube::{Move, MoveIterator},
    node_cube::node::{Node, Phase1Node, Phase2Node, Phase3Node},
    prune::{HashMapPruningTable, PruningTable},
};

pub trait Phase {
    const N_MOVES: usize;
    const MAX_DEPTH: u8;
    type PruningTable: PruningTable;
    type Node: Node;

    const N_STATES: usize = Self::Node::N_STATES;
    const MOVE_ITERATOR: MoveIterator = MoveIterator::new(Move(0)..Move(Self::N_MOVES as u8));
}

pub struct Phase1 {}

impl Phase for Phase1 {
    const N_MOVES: usize = 92;
    const MAX_DEPTH: u8 = 8;
    type PruningTable = HashMapPruningTable<Phase1>;
    type Node = Phase1Node;
}

pub struct Phase2 {}

impl Phase for Phase2 {
    const N_MOVES: usize = 44;
    const MAX_DEPTH: u8 = 10;
    type PruningTable = HashMapPruningTable<Phase2>;
    type Node = Phase2Node;
}

pub struct Phase3 {}

impl Phase for Phase3 {
    const N_MOVES: usize = 12;
    const MAX_DEPTH: u8 = 21;
    type PruningTable = HashMapPruningTable<Phase3>;

    type Node = Phase3Node;
}
