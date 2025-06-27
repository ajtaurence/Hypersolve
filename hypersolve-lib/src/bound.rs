use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver};
use std::sync::Arc;

use hypersolve_core::CubieCube;

use crate::{
    Cube, FastSolutionIterator, FixedLengthSolutionIterator, Node, Phase1Node,
    GODS_NUMBER_UPPER_BOUND,
};

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
    pub(crate) fn new(cube: Cube) -> Self {
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
