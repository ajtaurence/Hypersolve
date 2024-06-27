use hypersolve_core::{CubieCube, Move};

use crate::{Node, Phase1Node, Phase2Node, Phase3Node};

/// Returns a deterministic solution to the cube as fast as possible
pub fn simple_solve(cube: CubieCube) -> Vec<Move> {
    let phase1_sol = Phase1Node::from(cube)
        .phase_solutions(Vec::new(), ..)
        .next()
        .unwrap();

    let phase1_sol_len = phase1_sol.len();

    let phase2_cube = cube.apply_moves(phase1_sol.iter().copied());

    let phase2_sol = Phase2Node::from(phase2_cube)
        .phase_solutions(phase1_sol, ..)
        .next()
        .unwrap();

    let phase3_cube = phase2_cube.apply_moves(phase2_sol[phase1_sol_len..].iter().copied());

    Phase3Node::from(phase3_cube)
        .phase_solutions(phase2_sol, ..)
        .next()
        .unwrap()
}
