use hypersolve_core::Move;

use crate::Node;

/// Filters out moves that cannot solve the current node in `remaining_len` moves
pub(super) struct FilterSolveableNextMove<N, I> {
    current_node: N,
    next_move_iter: I,
    remaining_len: usize,
}

impl<N, I> FilterSolveableNextMove<N, I>
where
    N: Node,
    I: Iterator<Item = Move<N::Phase>>,
{
    pub fn new(current_node: N, next_move_iter: I, remaining_len: usize) -> Self {
        Self {
            next_move_iter,
            current_node,
            remaining_len,
        }
    }
}

impl<N, I> Iterator for FilterSolveableNextMove<N, I>
where
    N: Node,
    I: Iterator<Item = Move<N::Phase>>,
{
    type Item = (I::Item, N);
    fn next(&mut self) -> Option<Self::Item> {
        self.next_move_iter.find_map(|next_move| {
            let new_node = self.current_node.apply_move(next_move);

            let lower_bound = new_node.get_depth_bound() as usize;

            (self.remaining_len > lower_bound).then_some((next_move, new_node))
        })
    }
}
