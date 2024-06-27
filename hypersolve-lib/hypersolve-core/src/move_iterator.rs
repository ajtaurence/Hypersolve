use std::{iter::FusedIterator, marker::PhantomData, ops::Range};

use crate::*;

/// An iterator over the moves for a given phase
pub struct PhaseMoveIterator<P: Phase> {
    range: Range<u8>,
    _phase: PhantomData<P>,
}

impl<P: Phase> PhaseMoveIterator<P> {
    pub const fn new(is_last_move_of_phase: bool) -> Self {
        let mut start = 0;

        // If this is the last move of the phase then skip the moves in the next phase
        // because those moves cannot change the phase we are in
        if is_last_move_of_phase {
            if let Some(next_phase) = P::PHASE_ENUM.next_phase() {
                start = next_phase.n_moves() as u8;
            }
        }

        PhaseMoveIterator {
            range: start..P::N_MOVES as u8,
            _phase: PhantomData,
        }
    }
}

impl<P: Phase> Iterator for PhaseMoveIterator<P> {
    type Item = Move<P>;
    fn next(&mut self) -> Option<Self::Item> {
        // SAFTEY: i < P::N_MOVES
        self.range
            .next()
            .map(|i| unsafe { Move::from_u8_unchecked(i) })
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        // SAFTEY: i < P::N_MOVES
        self.range
            .nth(n)
            .map(|i| unsafe { Move::from_u8_unchecked(i) })
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.range.count()
    }
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        // SAFTEY: i < P::N_MOVES
        self.range
            .last()
            .map(|i| unsafe { Move::from_u8_unchecked(i) })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<P: Phase> DoubleEndedIterator for PhaseMoveIterator<P> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // SAFTEY: i < P::N_MOVES
        self.range
            .next_back()
            .map(|i| unsafe { Move::from_u8_unchecked(i) })
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        // SAFTEY: i < P::N_MOVES
        self.range
            .nth_back(n)
            .map(|i| unsafe { Move::from_u8_unchecked(i) })
    }
}

impl<P: Phase> FusedIterator for PhaseMoveIterator<P> {}

/// An iterator over moves
pub trait MoveIterator<P: Phase>: Iterator<Item = Move<P>> {
    /// Given a sequence of moves, it iterates over all non-redundant moves from phase `N` that could come next
    fn next_moves<N: Phase>(self, is_last_move_of_phase: bool) -> NextMoveIterator<P, N>
    where
        Self: Sized + DoubleEndedIterator,
    {
        NextMoveIterator::new(self, is_last_move_of_phase)
    }
}

impl<I, P: Phase> MoveIterator<P> for I where I: Iterator<Item = Move<P>> {}

/// An iterator over non-redundant next moves for a move sequence
pub struct NextMoveIterator<P: Phase, N: Phase> {
    last_moves: Option<(Move<P>, Option<Move<P>>)>,
    phase_move_iter: PhaseMoveIterator<N>,
}

impl<P: Phase, N: Phase> NextMoveIterator<P, N> {
    fn new(
        mut previous_moves: impl MoveIterator<P> + DoubleEndedIterator,
        is_last_move_of_phase: bool,
    ) -> Self {
        let last_moves = previous_moves
            .next_back()
            .map(|m| (m, previous_moves.next_back()));

        NextMoveIterator {
            last_moves,
            phase_move_iter: PhaseMoveIterator::<N>::new(is_last_move_of_phase),
        }
    }
}

impl<P: Phase, N: Phase> Iterator for NextMoveIterator<P, N> {
    type Item = Move<N>;
    fn next(&mut self) -> Option<Self::Item> {
        self.phase_move_iter
            .by_ref()
            .find(|&next_move| next_move_filter(self.last_moves, next_move) == KeepSkip::Keep)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.phase_move_iter.size_hint().1)
    }
}

impl<P: Phase, N: Phase> DoubleEndedIterator for NextMoveIterator<P, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(next_move) = self.phase_move_iter.next_back() {
            if next_move_filter(self.last_moves, next_move) == KeepSkip::Keep {
                return Some(next_move);
            }
        }
        None
    }
}

impl<P: Phase, N: Phase> FusedIterator for NextMoveIterator<P, N> {}

#[derive(Debug, Clone, Copy, PartialEq)]
enum KeepSkip {
    Keep,
    Skip,
}

/// Whether the next move should be kept or skipped given the last two
fn next_move_filter<P: Phase, N: Phase>(
    last_moves: Option<(Move<P>, Option<Move<P>>)>,
    next_move: Move<N>,
) -> KeepSkip {
    match last_moves {
        Some((last_move, last_last_move)) => {
            if next_move.axis() == last_move.axis() {
                KeepSkip::Skip
            } else {
                match last_last_move {
                    None => {
                        if next_move.into_u8() > last_move.into_u8()
                            && last_move.commutes_with(next_move)
                        {
                            KeepSkip::Skip
                        } else {
                            KeepSkip::Keep
                        }
                    }
                    Some(last_last_move) => {
                        if last_last_move.axis() == next_move.axis()
                            && last_move.commutes_with(last_last_move)
                        {
                            KeepSkip::Skip
                        } else {
                            KeepSkip::Keep
                        }
                    }
                }
            }
        }
        None => KeepSkip::Keep,
    }
}
