use super::*;

use crate::cubie_cube::MoveIterator;
use crate::phases::Phase;

pub struct NodeIterator<N: Node> {
    node: N,
    move_iter: MoveIterator,
}

impl<N: Node> NodeIterator<N> {
    pub const fn new(node: N) -> NodeIterator<N> {
        NodeIterator {
            node,
            move_iter: N::Phase::MOVE_ITERATOR,
        }
    }
}

impl<N: Node> Iterator for NodeIterator<N> {
    type Item = N;
    fn next(&mut self) -> Option<Self::Item> {
        let next_move = self.move_iter.next()?;

        if Some(next_move.axis()) == self.node.last_move().and_then(|m| Some(m.axis())) {
            return self.next();
        } else {
            return Some(self.node.apply_move(next_move));
        }
    }
}
