use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
};

use crate::{
    cubie_cube::{CubieCube, Move},
    node_cube::{node::Node, Phase1Node, Phase2Node, Phase3Node},
    piece_cube::{puzzle::PieceCube, twist::Twist},
};

/// Returns an iterator over all solutions of the specified length for the given node
fn phase_solutions_at_depth<N: Node + 'static>(
    node: N,
    sol_length: u32,
    sequence: Vec<Move>,
) -> Box<dyn Iterator<Item = Vec<Move>>> {
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

    // TODO: replace with axis priority so that node axes can cancel
    // For every connected node, try to find solutions from there
    Box::new(node.connected().flat_map(move |new_node| {
        let mut new_sequence = sequence.clone();
        new_sequence.push(new_node.last_move().unwrap());

        phase_solutions_at_depth(new_node, sol_length, new_sequence)
    }))
}

/// Iterates over all solutions for the node in order of increasing length
fn phase_solutions<N: Node + 'static>(node: N) -> impl Iterator<Item = Vec<Move>> {
    // find the minimum possible depth from the goal
    let min_depth = node.get_depth_bound();

    // start at the min depth and search longer solutions if needed
    (min_depth as u32..).flat_map(move |i| phase_solutions_at_depth(node, i, Vec::new()))
}

/// Solves the cube on this thread, sending solutions and solution lengths back via `solutions`
fn fast_solve_single_thread(
    cube: CubieCube,
    shortest_sol_length: Arc<AtomicU32>,
    solutions: Sender<(Vec<Twist>, u32)>,
    cube_rotation: Option<Twist>,
) {
    let phase1_cube = cube;

    for phase1_sol in phase_solutions::<Phase1Node>(cube.into()) {
        let phase1_sol_length = phase1_sol.len() as u32;

        // check if the solution will be longer than the shortest solution
        if phase1_sol_length >= shortest_sol_length.load(Ordering::Relaxed) {
            return;
        }

        let phase2_cube = phase1_cube.apply_moves(phase1_sol.clone());

        for phase2_sol in phase_solutions::<Phase2Node>(phase2_cube.into()) {
            // TODO: handle move cancelation
            let phase2_sol_length = phase2_sol.len() as u32 + phase1_sol_length;

            // check if the solution will be longer than the shortest solution
            if phase2_sol_length >= shortest_sol_length.load(Ordering::Relaxed) {
                break;
            }

            let phase3_cube = phase2_cube.apply_moves(phase2_sol.clone());

            let phase3_node = Phase3Node::from(phase3_cube);

            // if the lower bound on the solution length is equal to or longer than the shortest solution then don't bother to search it
            if phase2_sol_length + phase3_node.get_depth_bound() as u32 - 1 // -1 because we could have a move cancellation
                >= shortest_sol_length.load(Ordering::Relaxed)
            {
                break;
            }

            // get the phase 3 solution
            let phase3_sol = phase_solutions::<Phase3Node>(phase3_node).next().unwrap();

            // TODO: handle move cancelation
            let phase3_sol_length = phase3_sol.len() as u32 + phase2_sol_length;

            // store the shorter solution length in the atomic
            if shortest_sol_length.fetch_min(phase3_sol_length, Ordering::AcqRel)
                > phase3_sol_length
            {
                // if the value was swapped then this is the shortest solution

                // create the full solution
                let mut full_sol: Vec<_> = phase1_sol
                    .clone()
                    .into_iter()
                    .chain(phase2_sol.clone())
                    .chain(phase3_sol)
                    .map(|m| Twist::from(m))
                    .collect();
                if let Some(rotation) = cube_rotation {
                    full_sol.insert(0, rotation)
                }

                // send the solution
                let _ = solutions.send((full_sol, phase3_sol_length));
            }
        }
    }

    todo!()
}

/// Solves the given cube on multiple threads, sending solutions and solution lengths back as they are found
pub fn fast_solve(cube: PieceCube, max_sol_length: Option<u32>) -> Receiver<(Vec<Twist>, u32)> {
    let length = Arc::new(AtomicU32::new(max_sol_length.unwrap_or(u32::MAX)));

    let (raw_sol_send, raw_sol_receive) = channel();

    // spawn threads to search for solutions in parallel from different orientations
    thread::spawn(move || {
        // TODO: spawn a thread for each orientation
        fast_solve_single_thread(cube.into(), length.clone(), raw_sol_send.clone(), None);
    });

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
