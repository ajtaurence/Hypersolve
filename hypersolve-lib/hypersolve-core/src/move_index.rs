use std::marker::PhantomData;

use static_assertions::const_assert_eq;

use crate::*;

// Include constants generated at build-time
include!(concat!(env!("OUT_DIR"), "/const_gen.rs"));

const_assert_eq!(N_PHASE1_MOVES, 92);

/// Hypersolve move index for phases
///
/// # Invariant
/// The index is guaranteed to be less than `P::N_MOVES`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move<P: Phase = Phase1>(u8, PhantomData<P>);

impl<P: Phase> Move<P> {
    pub(crate) const unsafe fn from_u8_unchecked(index: u8) -> Self {
        Self(index, PhantomData)
    }

    pub const fn from_u8(index: u8) -> Self {
        assert!(index < P::N_MOVES as u8);
        // SAFTEY: we just checked
        unsafe { Self::from_u8_unchecked(index) }
    }

    pub const fn iter() -> PhaseMoveIterator<P> {
        PhaseMoveIterator::new(false)
    }

    #[inline(always)]
    pub const fn into_u8(self) -> u8 {
        // SAFTEY: this is the invariant
        unsafe { assert_unchecked!(self.0 < P::N_MOVES as u8) };
        self.0
    }

    #[inline(always)]
    pub const fn into_usize(self) -> usize {
        self.into_u8() as usize
    }

    /// Returns whether two moves commute with each other
    pub const fn commutes_with<T: Phase>(&self, other: Move<T>) -> bool {
        COMMUTATION_TABLE[self.into_usize()][other.into_usize()]
    }

    /// Returns the axis of this move
    pub const fn axis(&self) -> Axis {
        // Other implementation
        // match self.into_u8() {
        //     0 | 12..=17 | 44..=59 => Axis::X,
        //     1 | 18..=23 | 60..=75 => Axis::Y,
        //     2..=4 | 24..=27 | 76..=91 => Axis::Z,
        //     5..=11 | 28..=43 => Axis::W,
        //     _ => unsafe { std::hint::unreachable_unchecked() },
        // }

        AXIS_MOVE_TABLE[self.into_usize()]
    }

    /// Returns the twist object of this move
    pub const fn twist(&self) -> &Twist {
        &HYPERSOLVE_TWISTS[self.into_usize()]
    }

    /// Returns the piece permutation of this move
    pub const fn permutation(&self) -> &Permutation {
        &PERM_MOVE_TABLE[self.into_usize()]
    }

    /// Returns the piece orientation due this move
    pub const fn orientation(&self) -> &Orientation<A4> {
        &A4_MOVE_TABLE[self.into_usize()]
    }
}

impl From<Move<Phase3>> for Move<Phase1> {
    fn from(value: Move<Phase3>) -> Self {
        // SAFTEY: value < Phase3::N_MOVES < Phase1::N_MOVES
        unsafe { std::mem::transmute(value) }
    }
}

impl From<Move<Phase3>> for Move<Phase2> {
    fn from(value: Move<Phase3>) -> Self {
        // SAFTEY: value < Phase3::N_MOVES < Phase2::N_MOVES
        unsafe { std::mem::transmute(value) }
    }
}

impl From<Move<Phase2>> for Move<Phase1> {
    fn from(value: Move<Phase2>) -> Self {
        // SAFTEY: value < Phase2::N_MOVES < Phase1::N_MOVES
        unsafe { std::mem::transmute(value) }
    }
}
