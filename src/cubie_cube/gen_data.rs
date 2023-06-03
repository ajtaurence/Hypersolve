use super::*;

use crate::{groups::A4, phases::*, piece_cube::Twist};

const_data!(pub HYPERSOLVE_TWISTS: [Twist; Phase1::N_MOVES] =  gen_hypersolve_twists());
const_data!(pub PERM_MOVE_TABLE: [Permutation; Phase1::N_MOVES] =  gen_perm_move_table());
const_data!(pub A4_MOVE_TABLE: [Orientation<A4>; Phase1::N_MOVES] =  gen_a4_move_table());

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_hypersolve_twists() {
    let _ = &*HYPERSOLVE_TWISTS;
}

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_perm_move_table() {
    let _ = &*PERM_MOVE_TABLE;
}

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_a4_move_table() {
    let _ = &*A4_MOVE_TABLE;
}

/// Calculates the twists that hypersolve uses to solve the cube
#[cfg(feature = "gen-const-data")]
fn gen_hypersolve_twists() -> Box<[Twist; Phase1::N_MOVES]> {
    use crate::{
        groups::C3,
        piece_cube::{puzzle::PieceCube, PieceLocation},
    };
    use itertools::Itertools;
    // Generate twist which dont affect LDBO (index 15) and perform unique actions on a cube
    let twists = Twist::iter_all_twists()
        .filter(|&twist| !PieceLocation::from_index(15).is_affected_by_twist(twist))
        .unique_by(|&twist| PieceCube::solved().twist(twist))
        .collect_vec();

    // Order the twists by the phase
    let justphase1twists = twists
        .clone()
        .into_iter()
        .filter(|&twist| Orientation::<A4>::from(PieceCube::solved().twist(twist)).k4_coord() != 0);

    let justphase2twists = twists.clone().into_iter().filter(|&twist| {
        let cube = PieceCube::solved().twist(twist);
        Orientation::<A4>::from(cube).k4_coord() == 0
            && (Permutation::from(cube).io_coord() != 0
                || Orientation::<C3>::from(cube).c3_coord() != 0)
    });

    let justphase3twists = twists.into_iter().filter(|&twist| {
        let cube = PieceCube::solved().twist(twist);
        Orientation::<A4>::from(cube).k4_coord() == 0
            && Permutation::from(cube).io_coord() == 0
            && Orientation::<C3>::from(cube).c3_coord() == 0
    });

    // Combine all twists into a list in order of phase
    justphase3twists
        .chain(justphase2twists)
        .chain(justphase1twists)
        .collect_vec()
        .try_into()
        .unwrap()
}

/// Calculates the permutation move table using piece_cube
#[cfg(feature = "gen-const-data")]
fn gen_perm_move_table() -> Box<[Permutation; Phase1::N_MOVES]> {
    use crate::piece_cube::puzzle::PieceCube;
    use itertools::Itertools;
    HYPERSOLVE_TWISTS
        .iter()
        .map(|&twist| PieceCube::solved().twist(twist).into())
        .collect_vec()
        .try_into()
        .unwrap()
}

/// Calculates the A4 orientation move table using piece_cube
#[cfg(feature = "gen-const-data")]
fn gen_a4_move_table() -> Box<[Orientation<A4>; Phase1::N_MOVES]> {
    use crate::piece_cube::puzzle::PieceCube;
    use itertools::Itertools;
    HYPERSOLVE_TWISTS
        .iter()
        .map(|&twist| PieceCube::solved().twist(twist).into())
        .collect_vec()
        .try_into()
        .unwrap()
}
