use std::ops::RangeBounds;

use hypersolve_core::{Move, MoveIterator, NextMoveIterator, Phase1};

use crate::Node;

use super::next_move_filter::FilterSolveableNextMove;

#[allow(type_alias_bounds)]
type NextMoveIter<N: Node> = FilterSolveableNextMove<N, NextMoveIterator<Phase1, N::Phase>>;

pub struct PhaseSolutionIterator<N: Node, R> {
    stack: Vec<Move>,
    next_move_stack: Vec<NextMoveIter<N>>,
    total_sol_len: usize,
    sol_len_range: R,
    start_node: N,
    is_first_item: bool,
}

impl<N: Node, R: RangeBounds<usize>> PhaseSolutionIterator<N, R>
where
    Move<N::Phase>: Into<Move>,
{
    pub fn new(previous_sol: Vec<Move>, start_node: N, sol_len_range: R) -> Self {
        let total_sol_len = match sol_len_range.start_bound() {
            std::ops::Bound::Included(&inc) => inc,
            std::ops::Bound::Excluded(&exc) => exc + 1,
            std::ops::Bound::Unbounded => 0,
        }
        .max(previous_sol.len() + start_node.get_depth_bound() as usize);

        let mut stack = previous_sol;
        stack.reserve(total_sol_len - stack.len());
        let next_move_stack = Vec::with_capacity(total_sol_len - stack.len());

        Self {
            stack,
            next_move_stack,
            total_sol_len,
            sol_len_range,
            start_node,
            is_first_item: true,
        }
    }

    fn initialize_iter_stack(&mut self) -> Option<()> {
        if self.total_sol_len == self.stack.len() {
            debug_assert!(self.start_node == N::GOAL);
            return Some(());
        }

        let mut current_node = self.start_node;

        loop {
            let mut new_iter = NextMoveIter::new(
                current_node,
                self.stack
                    .iter()
                    .copied()
                    .next_moves(self.stack.len() + 1 == self.total_sol_len),
                self.total_sol_len - self.stack.len(),
            );

            if let Some((next_move, next_node)) = new_iter.next() {
                current_node = next_node;
                self.next_move_stack.push(new_iter);
                self.stack.push(next_move.into());

                if self.stack.len() == self.total_sol_len {
                    return Some(());
                }
            } else {
                current_node = self.prepare_next()?;
            }
        }
    }

    fn prepare_next(&mut self) -> Option<N> {
        if let Some(next_move_iter) = self.next_move_stack.last_mut() {
            if let Some((next_move, next_node)) = next_move_iter.next() {
                // prepare the stack with a new move
                self.stack.pop();
                self.stack.push(next_move.into());

                Some(next_node)
            } else {
                // no next move

                // decrease the stack length
                self.next_move_stack.pop();
                self.stack.pop();

                let is_last_move = self.stack.len() + 1 == self.total_sol_len;

                loop {
                    // the stack length is decreased so prepare the next subsequence if it exists
                    let new_node = self.prepare_next()?;

                    // Create a new iterator
                    let mut new_iter = NextMoveIter::new(
                        new_node,
                        self.stack.iter().copied().next_moves(is_last_move),
                        self.total_sol_len - self.stack.len(),
                    );

                    // If the iterator has a next move then push it to the stack and return success
                    if let Some((next_move, next_node)) = new_iter.next() {
                        self.next_move_stack.push(new_iter);
                        self.stack.push(next_move.into());
                        break Some(next_node);
                    }

                    // otherwise try again
                }
            }
        } else {
            // there are no iterators so we exhausted all move sequences
            None
        }
    }
}

impl<N: Node, R: RangeBounds<usize>> Iterator for PhaseSolutionIterator<N, R>
where
    Move<N::Phase>: Into<Move>,
{
    type Item = Vec<Move>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.sol_len_range.contains(&self.total_sol_len) {
            let cond = if self.is_first_item {
                self.is_first_item = false;

                self.initialize_iter_stack().is_some()
            } else {
                self.prepare_next().is_some()
            };

            if cond {
                return Some(self.stack.clone());
            } else {
                self.total_sol_len += 1;
                self.is_first_item = true;
                debug_assert!(self.next_move_stack.is_empty());
            }
        }

        None
    }
}
