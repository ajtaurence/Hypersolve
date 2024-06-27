use crate::*;

/// Solving phases
#[derive(Debug, Clone, Copy)]
pub enum PhaseEnum {
    Phase1,
    Phase2,
    Phase3,
}

impl PhaseEnum {
    /// Returns the next phase
    pub const fn next_phase(&self) -> Option<Self> {
        match self {
            PhaseEnum::Phase1 => Some(PhaseEnum::Phase2),
            PhaseEnum::Phase2 => Some(PhaseEnum::Phase3),
            PhaseEnum::Phase3 => None,
        }
    }

    /// Returns the previous phase
    pub const fn previous_phase(&self) -> Option<Self> {
        match self {
            PhaseEnum::Phase1 => None,
            PhaseEnum::Phase2 => Some(PhaseEnum::Phase1),
            PhaseEnum::Phase3 => Some(PhaseEnum::Phase2),
        }
    }

    /// Returns the number of moves available in this phase
    pub const fn n_moves(&self) -> usize {
        match self {
            PhaseEnum::Phase1 => Phase1::N_MOVES,
            PhaseEnum::Phase2 => Phase2::N_MOVES,
            PhaseEnum::Phase3 => Phase3::N_MOVES,
        }
    }
}

/// Solving phases
pub trait Phase: Copy {
    /// Number of moves available in this phase
    const N_MOVES: usize;
    /// God's number for this phase
    const MAX_DEPTH: usize;
    /// Phase as an enum
    const PHASE_ENUM: PhaseEnum;
}

#[derive(Debug, Clone, Copy)]
pub struct Phase1 {}
impl Phase for Phase1 {
    const N_MOVES: usize = N_PHASE1_MOVES;
    const MAX_DEPTH: usize = 8;
    const PHASE_ENUM: PhaseEnum = PhaseEnum::Phase1;
}

#[derive(Debug, Clone, Copy)]
pub struct Phase2 {}
impl Phase for Phase2 {
    const N_MOVES: usize = N_PHASE2_MOVES;
    const MAX_DEPTH: usize = 10;
    const PHASE_ENUM: PhaseEnum = PhaseEnum::Phase2;
}

#[derive(Debug, Clone, Copy)]
pub struct Phase3 {}
impl Phase for Phase3 {
    const N_MOVES: usize = N_PHASE3_MOVES;
    const MAX_DEPTH: usize = 21;
    const PHASE_ENUM: PhaseEnum = PhaseEnum::Phase3;
}
