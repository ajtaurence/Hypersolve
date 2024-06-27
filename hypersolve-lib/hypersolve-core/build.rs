use const_gen::*;
use hypersolve_base::*;
use itertools::Itertools;

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("const_gen.rs");

    let (n_jp1_moves, n_jp2_moves, n_jp3_moves, hypersolve_twists) = gen_hypersolve_twists();
    let perm_move_table = gen_perm_move_table(&hypersolve_twists);
    let a4_move_table = gen_a4_move_table(&hypersolve_twists);
    let axis_move_table = gen_axis_move_table(&hypersolve_twists);
    let commutation_table = gen_commutation_table(&hypersolve_twists);

    let const_declarations = [
        const_declaration!(pub(crate) N_PHASE1_MOVES = { n_jp1_moves + n_jp2_moves + n_jp3_moves }),
        const_declaration!(pub(crate) N_PHASE2_MOVES = { n_jp2_moves + n_jp3_moves }),
        const_declaration!(pub(crate) N_PHASE3_MOVES = n_jp3_moves),
        const_array_declaration!(HYPERSOLVE_TWISTS = hypersolve_twists),
        const_array_declaration!(PERM_MOVE_TABLE = perm_move_table),
        const_array_declaration!(A4_MOVE_TABLE = a4_move_table),
        const_array_declaration!(AXIS_MOVE_TABLE = axis_move_table),
        const_array_declaration!(COMMUTATION_TABLE = commutation_table),
    ]
    .join("\n");

    std::fs::write(dest_path, const_declarations).unwrap();
}

/// Calculates the twists that hypersolve uses to solve the cube and the number of moves in each phase
fn gen_hypersolve_twists() -> (usize, usize, usize, Vec<Twist>) {
    // Generate twist which dont affect LDBO (index 15) and perform unique actions on a cube
    let twists = Twist::ALL_TWISTS
        .into_iter()
        .filter(|twist| !PieceLocation::LAST.is_affected_by_twist(twist))
        .unique_by(|&twist| Cube::SOLVED.twist(twist))
        .collect_vec();

    // Order the twists by the phase
    let justphase1twists = twists
        .clone()
        .into_iter()
        .filter(|&twist| Orientation::<A4>::from_cube(Cube::SOLVED.twist(twist)).k4_coord() != 0);

    let justphase2twists = twists.clone().into_iter().filter(|&twist| {
        let cube = Cube::SOLVED.twist(twist);
        Orientation::<A4>::from_cube(cube).k4_coord() == 0
            && (Permutation::from_cube(cube).io_coord() != 0
                || Orientation::<C3>::from_cube(cube).c3_coord() != 0)
    });

    let justphase3twists = twists.into_iter().filter(|&twist| {
        let cube = Cube::SOLVED.twist(twist);
        Orientation::<A4>::from_cube(cube).k4_coord() == 0
            && Permutation::from_cube(cube).io_coord() == 0
            && Orientation::<C3>::from_cube(cube).c3_coord() == 0
    });

    // Combine all twists into a list in order of phase
    let hypersolve_twists = justphase3twists
        .clone()
        .chain(justphase2twists.clone())
        .chain(justphase1twists.clone())
        .collect_vec();

    (
        justphase1twists.collect_vec().len(),
        justphase2twists.collect_vec().len(),
        justphase3twists.collect_vec().len(),
        hypersolve_twists,
    )
}

/// Calculates the permutation move table using piece_cube
fn gen_perm_move_table(hypersolve_twists: &[Twist]) -> Vec<Permutation> {
    hypersolve_twists
        .iter()
        .map(|&twist| Permutation::from_cube(Cube::SOLVED.twist(twist)))
        .collect_vec()
}

/// Calculates the A4 orientation move table using piece_cube
fn gen_a4_move_table(hypersolve_twists: &[Twist]) -> Vec<Orientation<A4>> {
    hypersolve_twists
        .iter()
        .map(|&twist| Orientation::<A4>::from_cube(Cube::SOLVED.twist(twist)))
        .collect_vec()
}

/// Calculates the axes for each move in a lookup table
fn gen_axis_move_table(hypersolve_twists: &[Twist]) -> Vec<Axis> {
    hypersolve_twists
        .iter()
        .map(|twist| twist.face.axis())
        .collect_vec()
}

fn gen_commutation_table(hypersolve_twists: &[Twist]) -> Vec<Vec<bool>> {
    let mut result = vec![vec![false; hypersolve_twists.len()]; hypersolve_twists.len()];

    for (i, &twist_i) in hypersolve_twists.iter().enumerate() {
        for (j, &twist_j) in hypersolve_twists.iter().enumerate() {
            result[i][j] = Cube::SOLVED
                .twists([twist_i, twist_j, twist_i.inverse(), twist_j.inverse()])
                .is_solved()
        }
    }

    result
}
