use std::ops::RangeBounds;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver};
use std::sync::Arc;

use hypersolve_core::{CubieCube, Phase, Phase1, Phase2, Phase3};

use crate::simple_solve::simple_solve;
use crate::{Node, Phase1Node, Phase2Node, Phase3Node};

pub use crate::cube_index::{CubeIndex, CubeIndexError};
pub use crate::fast_solve::FastSolutionIterator;
pub use crate::solution_iterators::{FixedLengthSolutionIterator, ShortestSolutionIterator};
pub use hypersolve_core::{
    Notation, ParseMC4DTwistError, ParseStandardTwistError, ParseTwistError, Twist, TwistSequence,
};

/// Total number of 2<sup>4</sup> cube states (ignoring cube rotations)
pub const N_CUBE_STATES: u128 =
    Phase1Node::N_STATES as u128 * Phase2Node::N_STATES as u128 * Phase3Node::N_STATES as u128;

/// Upper bound on 2<sup>4</sup> God's number (STM)
///
/// A solution will always be found in this number of moves or less.
pub const GODS_NUMBER_UPPER_BOUND: usize =
    Phase1::MAX_DEPTH + Phase2::MAX_DEPTH + Phase3::MAX_DEPTH;

/// A 2<sup>4</sup> Rubik's Cube
#[derive(Clone, Copy, Default)]
pub struct Cube(hypersolve_core::Cube);

impl Cube {
    /// The solved cube
    pub const SOLVED: Self = Cube(hypersolve_core::Cube::SOLVED);

    /// Returns a new cube with the given twist applied to it
    pub fn twist(&self, twist: Twist) -> Self {
        Self(self.0.twist(twist))
    }

    /// Returns a new cube with the given twist sequence applied to it
    pub fn twist_seq(&self, twist_seq: impl IntoIterator<Item = Twist>) -> Self {
        let mut cube = *self;

        for twist in twist_seq {
            cube = cube.twist(twist);
        }

        cube
    }

    /// Deterministically finds a solution to the cube as quickly as possible
    ///
    /// The solution length is garanteed to be less than or equal to [`GODS_NUMBER_UPPER_BOUND`]
    pub fn fast_solve(&self) -> TwistSequence {
        simple_solve(CubieCube::from_cube(self.0))
            .into_iter()
            .map(|m| *m.twist())
            .collect()
    }

    /// Returns an iterator which non-deterministically returns increasingly shorter solutions
    pub fn fast_solutions(&self, max_solution_len: Option<usize>) -> FastSolutionIterator {
        FastSolutionIterator::new(self.0, max_solution_len)
    }

    /// Returns an iterator over all solutions to this cube in order of increasing length
    pub fn solutions(&self, solution_lengths: impl RangeBounds<usize>) -> ShortestSolutionIterator {
        ShortestSolutionIterator::new(CubieCube::from_cube(self.0), solution_lengths)
    }

    /// Returns an iterator over all solutions to this cube with the given length
    pub fn solutions_with_len(&self, solution_length: usize) -> FixedLengthSolutionIterator {
        FixedLengthSolutionIterator::new(CubieCube::from_cube(self.0), solution_length)
    }

    /// Returns an iterator over bounds on the length of the optimal solution
    pub fn optimal_bounds(&self) -> BoundIterator {
        BoundIterator::new(*self)
    }
}

/// Returns the scramble for the given cube index
pub fn new_scramble(cube_index: CubeIndex) -> TwistSequence {
    let cube = CubieCube::from(cube_index);

    simple_solve(cube).into_iter().map(|m| *m.twist()).collect()
}

enum BoundEnum<T> {
    Upper(T),
    Lower(T),
    Exact(T),
}

impl<T> BoundEnum<T> {
    fn is_exact(&self) -> bool {
        matches!(self, BoundEnum::Exact(_))
    }
}

struct LowerBoundIterator {
    cube: CubieCube,
    lower_bound: Option<usize>,
}

impl LowerBoundIterator {
    fn new(cube: Cube) -> Self {
        Self {
            cube: CubieCube::from_cube(cube.0),
            lower_bound: None,
        }
    }
}

impl Iterator for LowerBoundIterator {
    type Item = BoundEnum<usize>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(lower_bound) = self.lower_bound {
            if FixedLengthSolutionIterator::new(self.cube, lower_bound + 1)
                .next()
                .is_some()
            {
                Some(BoundEnum::Exact(lower_bound + 1))
            } else {
                self.lower_bound = Some(lower_bound + 1);
                Some(BoundEnum::Lower(lower_bound + 1))
            }
        } else {
            let bound = Phase1Node::from(self.cube).get_depth_bound() as usize;
            self.lower_bound = Some(bound);

            Some(BoundEnum::Lower(bound))
        }
    }
}

struct UpperBoundIterator {
    iter: FastSolutionIterator,
    last_bound: usize,
}

impl UpperBoundIterator {
    fn new(cube: Cube) -> Self {
        Self {
            iter: cube.fast_solutions(None),
            last_bound: GODS_NUMBER_UPPER_BOUND,
        }
    }
}

impl Iterator for UpperBoundIterator {
    type Item = BoundEnum<usize>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((_, len)) = self.iter.next() {
            self.last_bound = len;
            Some(BoundEnum::Upper(len))
        } else {
            Some(BoundEnum::Exact(self.last_bound))
        }
    }
}

pub struct BoundIterator {
    threads: Option<(std::thread::JoinHandle<()>, std::thread::JoinHandle<()>)>,
    work_flag: Arc<AtomicBool>,
    current_bound: Bound<usize>,
    rcv: Receiver<BoundEnum<usize>>,
}

impl BoundIterator {
    fn new(cube: Cube) -> Self {
        let (send, rcv) = sync_channel(0);

        let work_flag = Arc::new(AtomicBool::new(false));

        let c_work_flag = work_flag.clone();
        let c_send = send.clone();
        let lower_bound_thread = std::thread::spawn(move || {
            let mut iter = LowerBoundIterator::new(cube);

            loop {
                while !c_work_flag.load(Ordering::Relaxed) {
                    std::thread::park()
                }

                let bound = iter.next().unwrap();

                let cond = bound.is_exact();

                c_send.send(bound).unwrap();

                if cond {
                    return;
                }
            }
        });

        let c_work_flag = work_flag.clone();
        let c_send = send.clone();
        let upper_bound_thread = std::thread::spawn(move || {
            let mut iter = UpperBoundIterator::new(cube);

            loop {
                while !c_work_flag.load(Ordering::Relaxed) {
                    std::thread::park()
                }

                let bound = iter.next().unwrap();

                let cond = bound.is_exact();

                c_send.send(bound).unwrap();

                if cond {
                    return;
                }
            }
        });

        Self {
            threads: Some((lower_bound_thread, upper_bound_thread)),
            work_flag,
            current_bound: Bound {
                upper: GODS_NUMBER_UPPER_BOUND,
                lower: 0,
            },
            rcv,
        }
    }
}

impl Iterator for BoundIterator {
    type Item = Bound<usize>;
    fn next(&mut self) -> Option<Self::Item> {
        let threads = self.threads.as_mut()?;

        self.work_flag.store(true, Ordering::Release);

        threads.0.thread().unpark();
        threads.1.thread().unpark();

        let bound = self.rcv.recv().ok()?;

        match bound {
            BoundEnum::Upper(b) => self.current_bound.upper = b,
            BoundEnum::Lower(b) => self.current_bound.lower = b,
            BoundEnum::Exact(b) => {
                self.current_bound.upper = b;
                self.current_bound.lower = b;
            }
        }

        if self.current_bound.upper == self.current_bound.lower {
            // we are done
            self.threads.take();
        }

        self.work_flag.store(false, Ordering::Relaxed);

        Some(self.current_bound)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bound<T> {
    pub upper: T,
    pub lower: T,
}
