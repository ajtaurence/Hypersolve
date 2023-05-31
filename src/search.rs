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
    (0..).flat_map(move |i| phase_solutions_at_depth(node, i, Vec::new()))
}

/// Solves the cube on this thread, sending solutions back via `solutions`
fn fast_solve_single_thread(
    cube: CubieCube,
    shortest_sol_length: Arc<AtomicU32>,
    solutions: Sender<(Vec<Move>, Option<Twist>)>,
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

            'phase3: for phase3_sol in phase_solutions::<Phase3Node>(phase3_cube.into()) {
                // TODO: handle move cancelation
                let phase3_sol_length = phase3_sol.len() as u32 + phase2_sol_length;

                loop {
                    let current_shortest_length = shortest_sol_length.load(Ordering::SeqCst);

                    if phase3_sol_length >= current_shortest_length {
                        // The new length is not shorter, no need to update
                        break 'phase3;
                    }

                    // Attempt to update the length if it is still the same as the current length
                    match shortest_sol_length.compare_exchange_weak(
                        current_shortest_length,
                        phase3_sol_length,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    ) {
                        // Successfully updated the length
                        Ok(_) => {
                            // create the full solution
                            let full_sol = phase1_sol
                                .clone()
                                .into_iter()
                                .chain(phase2_sol.clone())
                                .chain(phase3_sol)
                                .collect();
                            // send the solution
                            let _ = solutions.send((full_sol, cube_rotation));
                            break;
                        }
                        // The length was updated by another thread, try again
                        Err(new_length) => {
                            if phase3_sol_length >= new_length {
                                // The new length is not shorter, no need to continue
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    todo!()
}

/// Solves the given cube on multiple threads, sending solutions back as they are found
pub fn fast_solve(cube: PieceCube, max_sol_length: Option<u32>) -> Receiver<Vec<Twist>> {
    let cubie = CubieCube::from(cube);

    let length = Arc::new(AtomicU32::new(max_sol_length.unwrap_or(u32::MAX)));

    let (raw_sol_send, raw_sol_receive) = channel();

    // spawn threads to search for solutions in parallel from different orientations
    thread::spawn(move || {
        // TODO: spawn a thread for each orientation
        fast_solve_single_thread(cubie, length, raw_sol_send, None);
    });

    // create a channel for converting solutions
    let (complete_sol_send, complete_sol_receive) = channel();

    // spawn a thread to simplify and convert incoming solutions
    thread::spawn(move || {
        while let Ok((solution, _cube_rotation)) = raw_sol_receive.recv() {
            // convert the solution to twist objects
            let converted_solution = solution.into_iter().map(|m| Twist::from(m)).collect();

            // TODO: simplify the solution and rotate it according to cube rotation before sending it back
            complete_sol_send.send(converted_solution).unwrap();
        }
    });

    complete_sol_receive
}
