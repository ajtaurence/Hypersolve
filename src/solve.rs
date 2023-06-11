use crate::{
    common::Face,
    cubie_cube::{CubieCube, Move},
    node_cube::{Node, Phase1Node, Phase2Node, Phase3Node},
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

/// Returns an iterator over all solutions of the specified length for the given node
fn phase_solutions_at_depth<N: Node + 'static>(
    node: N,
    sol_length: u32,
    sequence: Vec<Move>,
    //TODO: Rewrite this function using static dispatch
) -> Box<dyn Iterator<Item = Vec<Move>>> {
    let is_first_move = sequence.len() == 0;

    // If this is the goal node and the solution is the correct length then return the solution
    if node.is_goal() && sequence.len() as u32 == sol_length {
        return Box::new(std::iter::once(sequence.clone()));
    }

    // Lower bound on the number of moves required to solve the node
    let min_dist = sequence.len() as u8 + node.get_depth_bound();

    // If the minimum distance is more than the maximum depth then
    // return none since there is no reachable solution
    if min_dist as u32 > sol_length {
        return Box::new(std::iter::empty());
    }

    // For every connected node, try to find solutions from there
    if is_first_move {
        Box::new(node.connected_axis_priority().flat_map(move |new_node| {
            let mut new_sequence = sequence.clone();
            new_sequence.push(new_node.last_move().unwrap());

            phase_solutions_at_depth(new_node, sol_length, new_sequence)
        }))
    } else {
        Box::new(node.connected().flat_map(move |new_node| {
            let mut new_sequence = sequence.clone();
            new_sequence.push(new_node.last_move().unwrap());

            phase_solutions_at_depth(new_node, sol_length, new_sequence)
        }))
    }
}

/// Iterates over all solutions for the node in order of increasing length
fn phase_solutions<N: Node + 'static>(node: N) -> impl Iterator<Item = Vec<Move>> {
    // find the minimum possible depth from the goal
    let min_depth = node.get_depth_bound();

    // start at the min depth and search longer solutions if needed
    (min_depth as u32..).flat_map(move |i| phase_solutions_at_depth(node, i, Vec::new()))
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
    let phase1_sol = phase_solutions(Phase1Node::from(cube)).next().unwrap();

    cube = cube.apply_moves(phase1_sol.iter());

    let mut phase2_sol = phase_solutions(Phase2Node::from(cube)).next().unwrap();

    cube = cube.apply_moves(phase2_sol.iter());

    // TODO: sometimes this function isn't able to find a phase 3 solution for some reason?
    let mut phase3_sol = phase_solutions::<Phase3Node>(cube.into()).next().unwrap();

    let mut sequence = phase1_sol;
    sequence.append(&mut phase2_sol);
    sequence.append(&mut phase3_sol);

    TwistSequence::from(sequence).inverse()
}
