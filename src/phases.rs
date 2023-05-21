use crate::{
    cubiecube::cubiecube::{Move, MoveIterator},
    node_cube::node::{Node, Phase1Node, Phase2Node, Phase3Node},
};

pub trait Phase {
    const N_MOVES: usize;
    type Node: Node;

    const N_STATES: usize = Self::Node::N_STATES;
    const MOVE_ITERATOR: MoveIterator = MoveIterator::new(Move(0)..Move(Self::N_MOVES as u8));
}

pub struct Phase1 {}

impl Phase for Phase1 {
    const N_MOVES: usize = 92;
    type Node = Phase1Node;
}

pub struct Phase2 {}

impl Phase for Phase2 {
    const N_MOVES: usize = 44;
    type Node = Phase2Node;
}

pub struct Phase3 {}

impl Phase for Phase3 {
    const N_MOVES: usize = 12;
    type Node = Phase3Node;
}
