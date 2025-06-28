use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc,
    },
    thread::JoinHandle,
};

use hypersolve_core::{Cube, CubieCube, Face, Layer, Twist, TwistDirection, TwistSequence};

use crate::{Node, Phase1Node, Phase2Node, Phase3Node};

/// Solves the cube on this thread, sending solutions and solution lengths back via `solutions`
fn fast_solve(
    cube: CubieCube,
    search_flag: Arc<AtomicBool>,
    shortest_sol_length: Arc<AtomicUsize>,
    solutions: SyncSender<(TwistSequence, usize)>,
    pre_sequence: TwistSequence,
) {
    // if we should not be searching yet then park the thread
    while !search_flag.load(Ordering::Relaxed) {
        std::thread::park()
    }

    let phase1_cube = cube;

    for phase1_sol in Phase1Node::from(cube)
        .phase_solutions(Vec::new(), ..shortest_sol_length.load(Ordering::Relaxed))
    {
        let phase1_sol_len = phase1_sol.len();

        // check if the solution will be longer than the shortest solution
        if phase1_sol.len() >= shortest_sol_length.load(Ordering::Relaxed) {
            // Tell the threads to stop
            search_flag.store(false, Ordering::Relaxed);
            return;
        }

        let phase2_cube = phase1_cube.apply_moves(phase1_sol.iter().copied());

        for phase2_sol in Phase2Node::from(phase2_cube)
            .phase_solutions(phase1_sol, ..shortest_sol_length.load(Ordering::Relaxed))
        {
            // if we should not be searching then park the thread
            while !search_flag.load(Ordering::Relaxed) {
                std::thread::park()
            }

            // check if the solution will be longer than the shortest solution
            if phase2_sol.len() >= shortest_sol_length.load(Ordering::Relaxed) {
                break;
            }

            let phase3_cube = phase2_cube.apply_moves(phase2_sol[phase1_sol_len..].iter().copied());

            let phase3_node = Phase3Node::from(phase3_cube);

            let shortest_sol = shortest_sol_length.load(Ordering::Relaxed);

            if let Some(solution) = phase3_node
                .phase_solutions(phase2_sol, ..shortest_sol)
                .next()
            {
                // store the shorter solution length in the atomic
                if shortest_sol_length.fetch_min(solution.len(), Ordering::AcqRel) > solution.len()
                {
                    // if the value was swapped then this is the shortest solution

                    let sol_len = solution.len();

                    // map the solution into twists
                    let twists = solution.into_iter().map(|m| *m.twist());

                    // prepend the pre sequence
                    let twist_solution = pre_sequence.iter().copied().chain(twists).collect();

                    // send the solution
                    solutions.send((twist_solution, sol_len)).unwrap();
                }
            }
        }
    }
}

/// An iterator over increasingly shorter solutions
///
/// The iterator returns solutions and their corresponding lengths
pub struct FastSolutionIterator {
    thread_handles: [JoinHandle<()>; 12],
    sol_receive: Receiver<(TwistSequence, usize)>,
    search_flag: Arc<AtomicBool>,
}

impl FastSolutionIterator {
    pub(crate) fn new(cube: Cube, max_sol_length: Option<usize>) -> Self {
        let length = Arc::new(AtomicUsize::new(max_sol_length.unwrap_or(usize::MAX)));

        let (sol_send, sol_receive) = sync_channel(0);

        // Define all orientations that we want to search in parallel from.
        // This is all 12 orientations of the piece in the LDBO place
        let orientations = [
            Vec::new(),
            vec![Twist::new(Face::R, TwistDirection::DBR, Layer::Both)],
            vec![Twist::new(Face::R, TwistDirection::UFL, Layer::Both)],
            vec![Twist::new(Face::U, TwistDirection::UBL, Layer::Both)],
            vec![Twist::new(Face::U, TwistDirection::DFR, Layer::Both)],
            vec![Twist::new(Face::F, TwistDirection::DFL, Layer::Both)],
            vec![Twist::new(Face::F, TwistDirection::UBR, Layer::Both)],
            vec![Twist::new(Face::I, TwistDirection::DBL, Layer::Both)],
            vec![Twist::new(Face::I, TwistDirection::UFR, Layer::Both)],
            vec![
                Twist::new(Face::F, TwistDirection::DBR, Layer::Both),
                Twist::new(Face::I, TwistDirection::DBR, Layer::Both),
            ],
            vec![
                Twist::new(Face::F, TwistDirection::DFR, Layer::Both),
                Twist::new(Face::I, TwistDirection::DFR, Layer::Both),
            ],
            vec![
                Twist::new(Face::R, TwistDirection::UBR, Layer::Both),
                Twist::new(Face::I, TwistDirection::UBR, Layer::Both),
            ],
        ]
        .map(TwistSequence);

        let search_flag = Arc::new(AtomicBool::new(false));

        // spawn threads to search for solutions in parallel from different orientations
        let thread_handles = orientations.map(|twist_seq| {
            let c_length = length.clone();
            let c_raw_sol_send = sol_send.clone();
            let c_search_flag = search_flag.clone();
            let pre_seq = twist_seq.clone();

            std::thread::spawn(move || {
                fast_solve(
                    CubieCube::from_cube(cube.twists(twist_seq)),
                    c_search_flag,
                    c_length,
                    c_raw_sol_send,
                    pre_seq,
                );
            })
        });

        Self {
            thread_handles,
            sol_receive,
            search_flag,
        }
    }
}

impl Iterator for FastSolutionIterator {
    type Item = (TwistSequence, usize);
    fn next(&mut self) -> Option<Self::Item> {
        // If any of the threads have finished then all remaining bounds are in the receiver (if any)
        if self
            .thread_handles
            .iter()
            .any(|thread| thread.is_finished())
        {
            return self.sol_receive.try_recv().ok();
        }

        // Otherwise tell the threads to start searching
        self.search_flag.store(true, Ordering::Release);

        // Unpark all the threads
        for thread in &self.thread_handles {
            thread.thread().unpark()
        }

        // wait for a solution
        let result = self.sol_receive.recv().ok();

        // tell the threads to stop searching
        self.search_flag.store(false, Ordering::Relaxed);

        result
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_fast_solve() {
        let cube = Cube::SOLVED.twists(TwistSequence::from_str("IUFL UFLI BR FURI").unwrap());

        let sol_iter = FastSolutionIterator::new(cube, None);

        for (sol, _) in sol_iter {
            assert!(cube.twists(sol.iter().copied()).is_solved());
        }
    }
}
