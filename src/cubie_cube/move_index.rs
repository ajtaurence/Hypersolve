use crate::{common::Axis, node_cube::node::MOVE_AXIS};

/// Hypersolve move index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move(pub u8);

impl std::ops::Deref for Move {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Move {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Move {
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn axis(&self) -> Axis {
        MOVE_AXIS[self.as_usize()]
    }
}

pub struct MoveIterator {
    moves: std::ops::Range<Move>,
    current_move: Move,
}

impl MoveIterator {
    pub const fn new(moves: std::ops::Range<Move>) -> Self {
        Self {
            moves,
            current_move: Move(0),
        }
    }
}

impl Iterator for MoveIterator {
    type Item = Move;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_move == self.moves.end {
            return None;
        } else {
            let result = self.current_move;
            self.current_move = Move(self.current_move.as_u8() + 1);
            return Some(result);
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.moves.end.as_usize() - self.current_move.as_usize();
        (remaining, Some(remaining))
    }
}
