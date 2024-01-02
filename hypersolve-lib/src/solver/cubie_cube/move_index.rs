use std::marker::PhantomData;

use crate::puzzle::Axis;

use super::*;

/// Hypersolve move index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move(pub u8);

impl From<Move> for crate::puzzle::Twist {
    fn from(value: Move) -> Self {
        HYPERSOLVE_TWISTS[value.0 as usize]
    }
}

impl From<Vec<Move>> for crate::puzzle::TwistSequence {
    fn from(value: Vec<Move>) -> Self {
        value.into_iter().map(|m| m.into()).collect()
    }
}

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

/// An iterator over a range of moves
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
            None
        } else {
            let result = self.current_move;
            self.current_move = Move(self.current_move.as_u8() + 1);
            Some(result)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.moves.end.as_usize() - self.current_move.as_usize();
        (remaining, Some(remaining))
    }
}

const_data!(
    AXIS_SORTED_MOVES: [[[Move; 92]; 4]; 3] = {
        use itertools::Itertools;
        use strum::IntoEnumIterator;

        // indexed by [phase][axis][move]
        let mut result = [[[Move(0); 92]; 4]; 3];

        for (phase, entry) in result.iter_mut().enumerate() {
            let n_moves = match phase {
                0 => Phase1::N_MOVES,
                1 => Phase2::N_MOVES,
                2 => Phase3::N_MOVES,
                _ => unreachable!(),
            } as u8;

            for axis in Axis::iter() {
                for (i, value) in MoveIterator::new(Move(0)..Move(n_moves))
                    .sorted_by_key(|m| (m.axis() as i8 - axis as i8).abs())
                    .enumerate()
                {
                    *entry[axis as usize][i] = *value;
                }
            }
        }

        Box::new(result)
    }
);

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_axis_sorted_moves() {
    let _ = *AXIS_SORTED_MOVES;
}

/// An iterator over a set of moves that don't contain moves along a certain axis
pub(crate) struct AxisPriorityMoveIterator<P: Phase> {
    axis_sorted_moves: std::slice::Iter<'static, Move>,
    phantom: PhantomData<P>,
}

impl<P: Phase> AxisPriorityMoveIterator<P> {
    pub fn new(axis: Axis) -> Self {
        Self {
            axis_sorted_moves: AXIS_SORTED_MOVES[P::PHASE_INDEX][axis as usize][..P::N_MOVES]
                .iter(),
            phantom: PhantomData,
        }
    }
}

impl<P: Phase> Iterator for AxisPriorityMoveIterator<P> {
    type Item = Move;
    fn next(&mut self) -> Option<Self::Item> {
        self.axis_sorted_moves.next().copied()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.axis_sorted_moves.size_hint()
    }
}
