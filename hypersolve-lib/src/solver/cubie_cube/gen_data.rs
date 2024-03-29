use super::*;

use crate::puzzle::Twist;

#[test]
fn generate_hypersolve_twists() {
    let _ = &*HYPERSOLVE_TWISTS;
}

#[test]
fn generate_perm_move_table() {
    let _ = &*PERM_MOVE_TABLE;
}

#[test]
fn generate_a4_move_table() {
    let _ = &*A4_MOVE_TABLE;
}

/// Calculates the twists that hypersolve uses to solve the cube
pub(super) fn gen_hypersolve_twists() -> Box<[Twist; Phase1::N_MOVES]> {
    use crate::common::groups::{A4, C3};
    use crate::puzzle::*;

    use itertools::Itertools;
    // Generate twist which dont affect LDBO (index 15) and perform unique actions on a cube
    let twists = Twist::iter()
        .filter(|&twist| {
            !PieceLocation::from_index(PieceLocationIndex(15)).is_affected_by_twist(twist)
        })
        .unique_by(|&twist| Cube::solved().twist(twist))
        .collect_vec();

    // Order the twists by the phase
    let justphase1twists = twists
        .clone()
        .into_iter()
        .filter(|&twist| Orientation::<A4>::from(Cube::solved().twist(twist)).k4_coord() != 0);

    let justphase2twists = twists.clone().into_iter().filter(|&twist| {
        let cube = Cube::solved().twist(twist);
        Orientation::<A4>::from(cube).k4_coord() == 0
            && (Permutation::from(cube).io_coord() != 0
                || Orientation::<C3>::from(cube).c3_coord() != 0)
    });

    let justphase3twists = twists.into_iter().filter(|&twist| {
        let cube = Cube::solved().twist(twist);
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

pub(super) fn gen_perm_move_table() -> Box<[Permutation; Phase1::N_MOVES]> {
    use crate::puzzle::Cube;
    use itertools::Itertools;
    HYPERSOLVE_TWISTS
        .iter()
        .map(|&twist| Cube::solved().twist(twist).into())
        .collect_vec()
        .try_into()
        .unwrap()
}

/// Calculates the A4 orientation move table using piece_cube
pub(super) fn gen_a4_move_table() -> Box<[Orientation<crate::common::groups::A4>; Phase1::N_MOVES]>
{
    use crate::puzzle::Cube;
    use itertools::Itertools;
    HYPERSOLVE_TWISTS
        .iter()
        .map(|&twist| Cube::solved().twist(twist).into())
        .collect_vec()
        .try_into()
        .unwrap()
}
