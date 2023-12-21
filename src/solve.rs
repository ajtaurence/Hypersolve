use crate::{
    common::Face,
    cubie_cube::{CubieCube, Move},
    node_cube::{
        ConnectedNodeIterator, Node, NodeAxisFilterIterator, NodeAxisPriorityIterator, Phase1Node,
        Phase2Node, Phase3Node,
    },
    piece_cube::{puzzle::PieceCube, LayerEnum, Twist, TwistDirectionEnum, TwistSequence},
};
use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
};

struct FixedLengthPhaseSolutionIterator<N: Node, I: ConnectedNodeIterator<N>> {
    node: N,
    sol_length: u32,
    node_iter: I,
    sub_sol_iter: Option<Box<FixedLengthPhaseSolutionIterator<N, NodeAxisFilterIterator<N>>>>,
    last_move: Option<Move>,
}

impl<N: Node, I: ConnectedNodeIterator<N>> FixedLengthPhaseSolutionIterator<N, I> {
    fn new(node: N, sol_length: u32) -> Self {
        let mut node_iter: I = node.connected();

        let mut last_move = None;

        let sub_sol_iter = if sol_length == 0 {
            None
        } else {
            node_iter.next().map(|n| {
                last_move = n.last_move();
                Box::new(FixedLengthPhaseSolutionIterator::new(n, sol_length - 1))
            })
        };

        Self {
            node,
            sol_length,
            node_iter,
            sub_sol_iter,
            last_move,
        }
    }
}

impl<N: Node, I: ConnectedNodeIterator<N>> Iterator for FixedLengthPhaseSolutionIterator<N, I> {
    type Item = Vec<Move>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.node.get_depth_bound() as u32 > self.sol_length {
            return None;
        }

        if let Some(ref mut sub_sol_iter) = self.sub_sol_iter {
            // There is a sub_sol_iter

            if let Some(sub_sol) = sub_sol_iter.next() {
                // We found a sub solution

                // Add the last move to the solution and return it
                return Some([vec![self.last_move.unwrap()], sub_sol].concat());
            } else {
                // There are no more solutions in this sub iterator

                // Get the next node
                if let Some(next_node) = self.node_iter.next() {
                    self.last_move = next_node.last_move();

                    // create a new subsolution
                    self.sub_sol_iter = Some(Box::new(FixedLengthPhaseSolutionIterator::new(
                        next_node,
                        self.sol_length - 1,
                    )));

                    // Recursively call the function
                    return self.next();
                } else {
                    // There is no next node, we are done
                    return None;
                }
            }
        } else {
            // There is no sub_sol_iter because the solution length is zero

            if self.node.is_goal() {
                // Return empty solution if already solved
                return Some(Vec::new());
            } else {
                return None;
            }
        }
    }
}

/// Iterates over all solutions for the node in order of increasing length
fn phase_solutions<N: Node + 'static>(node: N) -> impl Iterator<Item = Vec<Move>> {
    // find the minimum possible depth from the goal
    let min_depth = node.get_depth_bound();

    // start at the min depth and search longer solutions if needed
    (min_depth as u32..).flat_map(move |i| {
        FixedLengthPhaseSolutionIterator::<N, NodeAxisPriorityIterator<N>>::new(node, i)
    })
}

/// Solves the cube on this thread, sending solutions and solution lengths back via `solutions`
fn fast_solve_internal(
    cube: CubieCube,
    shortest_sol_length: Arc<AtomicU32>,
    solutions: Sender<(TwistSequence, u32)>,
    pre_sequence: TwistSequence,
) {
    let phase1_cube = cube;

    for phase1_sol in phase_solutions::<Phase1Node>(cube.into()) {
        let phase1_sol_length = phase1_sol.len() as u32;

        // check if the solution will be longer than the shortest solution
        if phase1_sol_length >= shortest_sol_length.load(Ordering::Relaxed) {
            return;
        }

        let phase2_cube = phase1_cube.apply_moves(phase1_sol.iter());
        let mut phase2_node: Phase2Node = phase2_cube.into();
        phase2_node.last_move = phase1_sol.last().copied();

        for phase2_sol in phase_solutions::<Phase2Node>(phase2_node) {
            let phase2_move_cancel = if phase2_sol
                .first()
                .map(|m| Some(m.axis()) == phase2_node.last_axis())
                .unwrap_or(false)
            {
                1
            } else {
                0
            };

            let phase2_sol_length =
                phase2_sol.len() as u32 + phase1_sol_length - phase2_move_cancel;

            // check if the solution will be longer than the shortest solution
            if phase2_sol_length >= shortest_sol_length.load(Ordering::Relaxed) {
                break;
            }

            let phase3_cube = phase2_cube.apply_moves(phase2_sol.iter());

            let mut phase3_node = Phase3Node::from(phase3_cube);
            phase3_node.last_move = phase2_sol.last().copied();

            // if the lower bound on the solution length is equal to or longer than the shortest solution then don't bother to search it
            if phase2_sol_length + phase3_node.get_depth_bound() as u32
                > shortest_sol_length.load(Ordering::Relaxed)
            {
                break;
            }

            // get the phase 3 solution
            let phase3_sol = phase_solutions::<Phase3Node>(phase3_node).next().unwrap();

            let phase3_move_cancel = if phase3_sol
                .first()
                .map(|m| Some(m.axis()) == phase3_node.last_axis())
                .unwrap_or(false)
            {
                1
            } else {
                0
            };

            let phase3_sol_length =
                phase3_sol.len() as u32 + phase2_sol_length - phase3_move_cancel;

            // store the shorter solution length in the atomic
            if shortest_sol_length.fetch_min(phase3_sol_length, Ordering::AcqRel)
                > phase3_sol_length
            {
                // if the value was swapped then this is the shortest solution

                // create the solution
                let mut sol: Vec<_> = phase1_sol
                    .clone()
                    .into_iter()
                    .chain(phase2_sol.clone())
                    .chain(phase3_sol)
                    .map(Twist::from)
                    .collect();

                // add the pre sequence
                let mut full_sol = pre_sequence.clone();
                full_sol.0.append(&mut sol);

                // send the solution
                solutions.send((full_sol, phase3_sol_length)).unwrap();
            }
        }
    }
}

/// Solves the given cube on multiple threads, sending solutions and solution lengths back as they are found
pub fn fast_solve(cube: PieceCube, max_sol_length: Option<u32>) -> Receiver<(TwistSequence, u32)> {
    let length = Arc::new(AtomicU32::new(max_sol_length.unwrap_or(u32::MAX)));

    let (raw_sol_send, raw_sol_receive) = channel();

    // Define all orientations that we want to search in parallel from.
    // This is all 12 orientations of the piece in the LDBO place
    let orientations = [
        Vec::new(),
        vec![Twist::new(
            Face::R,
            TwistDirectionEnum::DBL,
            LayerEnum::Both,
        )],
        vec![Twist::new(
            Face::R,
            TwistDirectionEnum::UFR,
            LayerEnum::Both,
        )],
        vec![Twist::new(
            Face::U,
            TwistDirectionEnum::DBL,
            LayerEnum::Both,
        )],
        vec![Twist::new(
            Face::U,
            TwistDirectionEnum::UFR,
            LayerEnum::Both,
        )],
        vec![Twist::new(
            Face::F,
            TwistDirectionEnum::DBL,
            LayerEnum::Both,
        )],
        vec![Twist::new(
            Face::F,
            TwistDirectionEnum::UFR,
            LayerEnum::Both,
        )],
        vec![Twist::new(
            Face::I,
            TwistDirectionEnum::DBL,
            LayerEnum::Both,
        )],
        vec![Twist::new(
            Face::I,
            TwistDirectionEnum::UFR,
            LayerEnum::Both,
        )],
        vec![
            Twist::new(Face::F, TwistDirectionEnum::DBL, LayerEnum::Both),
            Twist::new(Face::I, TwistDirectionEnum::DBL, LayerEnum::Both),
        ],
        vec![
            Twist::new(Face::F, TwistDirectionEnum::UFR, LayerEnum::Both),
            Twist::new(Face::I, TwistDirectionEnum::UFR, LayerEnum::Both),
        ],
        vec![
            Twist::new(Face::R, TwistDirectionEnum::DBL, LayerEnum::Both),
            Twist::new(Face::I, TwistDirectionEnum::DBL, LayerEnum::Both),
        ],
    ]
    .map(|v| TwistSequence(v));

    // spawn threads to search for solutions in parallel from different orientations
    for twist_seq in orientations {
        let c_length = length.clone();
        let c_raw_sol_send = raw_sol_send.clone();
        let pre_seq = twist_seq.clone();

        thread::spawn(move || {
            fast_solve_internal(
                cube.twists(twist_seq).into(),
                c_length,
                c_raw_sol_send,
                pre_seq,
            );
        });
    }

    // create a channel for converting solutions
    let (complete_sol_send, complete_sol_receive) = channel();

    // spawn a thread to simplify incoming solutions
    thread::spawn(move || {
        while let Ok((solution, length)) = raw_sol_receive.recv() {
            // TODO: simplify the solution and rotate it according to cube rotation before sending it back
            complete_sol_send.send((solution, length)).unwrap();
        }
    });

    complete_sol_receive
}

/// Finds a scramble that results in the given cube
pub fn find_scramble(mut cube: CubieCube) -> TwistSequence {
    // Find phase 1 solution
    let phase1_sol = phase_solutions(Phase1Node::from(cube)).next().unwrap();

    // Apply phase 1 solution
    cube = cube.apply_moves(phase1_sol.iter());

    // Find phase 2 solution
    let mut phase2_sol = phase_solutions(Phase2Node::from(cube)).next().unwrap();

    // Apply phase 2 solution
    cube = cube.apply_moves(phase2_sol.iter());

    // Find phase 3 solution
    let mut phase3_sol = phase_solutions(Phase3Node::from(cube)).next().unwrap();

    let mut sequence = phase1_sol;
    sequence.append(&mut phase2_sol);
    sequence.append(&mut phase3_sol);

    TwistSequence::from(sequence).inverse()
}
