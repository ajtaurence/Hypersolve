use std::ops::RangeBounds;

use hypersolve_core::{CubieCube, TwistSequence};

use super::fixed_length_solution_iterator::FixedLengthSolutionIterator;

/// An iterator over solutions to a cube in order of increasing length
pub struct ShortestSolutionIterator {
    sol_len_limit: usize,
    fixed_len_iter: FixedLengthSolutionIterator,
}

impl ShortestSolutionIterator {
    pub(crate) fn new(cube: CubieCube, solution_lengths: impl RangeBounds<usize>) -> Self {
        let min_sol_len = match solution_lengths.start_bound() {
            std::ops::Bound::Included(&inc) => inc,
            std::ops::Bound::Excluded(&exc) => exc + 1,
            std::ops::Bound::Unbounded => 0,
        };

        let sol_len_limit = match solution_lengths.end_bound() {
            std::ops::Bound::Included(&inc) => inc + 1,
            std::ops::Bound::Excluded(&exc) => exc,
            std::ops::Bound::Unbounded => usize::MAX,
        };

        Self {
            sol_len_limit,
            fixed_len_iter: FixedLengthSolutionIterator::new(cube, min_sol_len),
        }
    }

    /// Sets the iterator to begin outputing solutions of the given length
    pub fn set_to_length(&mut self, solution_length: usize) {
        self.fixed_len_iter.reset_to_len(solution_length)
    }
}

impl Iterator for ShortestSolutionIterator {
    type Item = TwistSequence;
    fn next(&mut self) -> Option<Self::Item> {
        if self.fixed_len_iter.sol_len() >= self.sol_len_limit {
            return None;
        }

        loop {
            if let Some(sol) = self.fixed_len_iter.next() {
                return Some(sol);
            } else {
                let sol_len = self.fixed_len_iter.sol_len();
                if sol_len + 1 < self.sol_len_limit {
                    self.fixed_len_iter.reset_to_len(sol_len + 1);
                    continue;
                } else {
                    return None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use hypersolve_core::{Cube, CubieCube, Notation};

    use super::ShortestSolutionIterator;

    #[test]
    fn test_optimal_solution() {
        let cube = CubieCube::from_cube(
            Cube::SOLVED.twists(
                Notation::Standard
                    .parse_twist_sequence("RO2 UF2 IF2 FR2")
                    .unwrap()
                    .inverse(),
            ),
        );

        let mut sols = ShortestSolutionIterator::new(cube, ..=3);

        // no solutions with length 3 or less
        assert!(sols.next().is_none());

        // solutions should be found with length 4
        let mut sols = ShortestSolutionIterator::new(cube, ..=4);

        assert!(sols.next().is_some())
    }
}
