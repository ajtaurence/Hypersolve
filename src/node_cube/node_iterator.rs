use super::*;

use crate::cubie_cube::{AxisPriorityMoveIterator, MoveIterator};
use crate::phases::Phase;

/// An iterator over nodes connected to a given node by the available moves.
/// Ignores moves on the same axis as the previous move.
pub struct NodeAxisFilterIterator<N: Node> {
    node: N,
    move_iter: MoveIterator,
}

impl<N: Node> NodeAxisFilterIterator<N> {
    pub const fn new(node: N) -> NodeAxisFilterIterator<N> {
        NodeAxisFilterIterator {
            node,
            move_iter: N::Phase::MOVE_ITERATOR,
        }
    }
}

impl<N: Node> Iterator for NodeAxisFilterIterator<N> {
    type Item = N;
    fn next(&mut self) -> Option<Self::Item> {
        let next_move = self.move_iter.next()?;

        if Some(next_move.axis()) == self.node.last_move().map(|m| m.axis()) {
            self.next()
        } else {
            Some(self.node.apply_move(next_move))
        }
    }
}

/// An iterator over nodes connected to a given node by the available moves.
/// Moves on axes that match the last move axis will be returned first
pub struct NodeAxisPriorityIterator<N: Node> {
    node: N,
    move_iter: AxisPriorityMoveIterator<N::Phase>,
}

impl<N: Node> NodeAxisPriorityIterator<N> {
    pub fn new(node: N) -> NodeAxisPriorityIterator<N> {
        let move_iter = AxisPriorityMoveIterator::<N::Phase>::new(
            node.last_move().map(|m| m.axis()).unwrap_or_default(),
        );

        NodeAxisPriorityIterator { node, move_iter }
    }
}

impl<N: Node> Iterator for NodeAxisPriorityIterator<N> {
    type Item = N;
    fn next(&mut self) -> Option<Self::Item> {
        self.move_iter.next().map(|m| self.node.apply_move(m))
    }
}
