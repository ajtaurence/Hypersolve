use std::ops::{RangeInclusive, RangeToInclusive};

use hypersolve_core::{CubieCube, Move, TwistSequence};

use crate::{Node, Phase1Node, Phase2Node, Phase3Node};

use super::phase_solution_iterator::PhaseSolutionIterator;

struct Phase23SolIter {
    total_sol_len: usize,
    phase2_sol_iter: PhaseSolutionIterator<Phase2Node, RangeToInclusive<usize>>,
    phase1_solved_cube: CubieCube,
    phase1_sol_len: usize,
    phase3_sol_iter: Option<PhaseSolutionIterator<Phase3Node, RangeInclusive<usize>>>,
}

impl Phase23SolIter {
    fn new(previous_sol: Vec<Move>, start_cube: CubieCube, total_sol_len: usize) -> Self {
        let phase1_sol_len = previous_sol.len();

        Self {
            total_sol_len,
            phase2_sol_iter: Phase2Node::from(start_cube)
                .phase_solutions(previous_sol, ..=total_sol_len),
            phase1_solved_cube: start_cube,
            phase1_sol_len,
            phase3_sol_iter: None,
        }
    }
}

impl Iterator for Phase23SolIter {
    type Item = Vec<Move>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(p3_sol_iter) = self.phase3_sol_iter.as_mut() {
                if let Some(sol) = p3_sol_iter.next() {
                    return Some(sol);
                }
            }

            let p2_sol = self.phase2_sol_iter.next()?;

            let p2_solved_cube = self
                .phase1_solved_cube
                .apply_moves(p2_sol[self.phase1_sol_len..].iter().copied());

            self.phase3_sol_iter = Some(
                Phase3Node::from(p2_solved_cube)
                    .phase_solutions(p2_sol, self.total_sol_len..=self.total_sol_len),
            );
        }
    }
}

pub struct FixedLengthSolutionIterator {
    solution_length: usize,
    init_cube: CubieCube,
    phase1_sol_iter: PhaseSolutionIterator<Phase1Node, RangeToInclusive<usize>>,
    phase23_sol_iter: Option<Phase23SolIter>,
}

impl FixedLengthSolutionIterator {
    pub(crate) fn new(cube: CubieCube, solution_length: usize) -> Self {
        let phase1_sol_iter =
            Phase1Node::from(cube).phase_solutions(Vec::new(), ..=solution_length);

        Self {
            solution_length,
            init_cube: cube,
            phase1_sol_iter,
            phase23_sol_iter: None,
        }
    }

    /// Gets the solution length
    pub fn sol_len(&self) -> usize {
        self.solution_length
    }

    /// Resets the iterator to the given length
    pub fn reset_to_len(&mut self, solution_length: usize) {
        self.solution_length = solution_length;

        self.phase1_sol_iter =
            Phase1Node::from(self.init_cube).phase_solutions(Vec::new(), ..=solution_length);

        self.phase23_sol_iter = None;
    }
}

impl Iterator for FixedLengthSolutionIterator {
    type Item = TwistSequence;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(p23_sol_iter) = self.phase23_sol_iter.as_mut() {
                if let Some(sol) = p23_sol_iter.next() {
                    return Some(sol.into_iter().map(|m| *m.twist()).collect());
                }
            }

            let p1_sol = self.phase1_sol_iter.next()?;

            let p1_solved_cube = self.init_cube.apply_moves(p1_sol.iter().copied());

            self.phase23_sol_iter = Some(Phase23SolIter::new(
                p1_sol,
                p1_solved_cube,
                self.solution_length,
            ));
        }
    }
}
