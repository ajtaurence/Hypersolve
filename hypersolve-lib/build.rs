use const_gen::*;
use hypersolve_core::*;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("const_gen.rs");

    let io_move_table = gen_io_move_table();
    let i_move_table = gen_i_move_table();
    let o_move_table = gen_o_move_table();

    let const_declarations = [
        static_array_declaration!(IO_MOVE_TABLE = io_move_table),
        static_array_declaration!(I_MOVE_TABLE = i_move_table),
        static_array_declaration!(O_MOVE_TABLE = o_move_table),
    ]
    .join("\n");

    std::fs::write(dest_path, const_declarations).unwrap();
}

fn gen_io_move_table() -> Vec<[u16; Phase2::N_MOVES]> {
    let mut table = vec![[0_u16; Phase2::N_MOVES]; N_IO_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        let cube = CubieCube {
            orientation: Orientation::SOLVED,
            permutation: Permutation::from_coords(i as u16, 0, 0),
        };

        for (j, val) in entry.iter_mut().enumerate() {
            *val = cube
                .apply_move(Move::<Phase2>::from_u8(j as u8))
                .permutation
                .io_coord();
        }
    });

    table
}

fn gen_i_move_table() -> Vec<[u16; Phase3::N_MOVES]> {
    let mut table = vec![[0_u16; Phase3::N_MOVES]; N_I_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        // Ensure the total permutation parity of the cube remains valid
        let o_coord = if i < N_I_COORD_STATES as usize / 2 {
            0
        } else {
            N_O_COORD_STATES / 2
        };

        let cube = CubieCube {
            orientation: Orientation::SOLVED,
            permutation: Permutation::from_coords(0, i as u16, o_coord),
        };

        for (j, val) in entry.iter_mut().enumerate() {
            *val = cube
                .apply_move(Move::<Phase3>::from_u8(j as u8))
                .permutation
                .i_coord();
        }
    });

    table
}

fn gen_o_move_table() -> Vec<[u16; Phase3::N_MOVES]> {
    let mut table = vec![[0_u16; Phase3::N_MOVES]; N_O_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        // Ensure the total permutation parity of the cube remains valid
        let i_coord = if i < N_O_COORD_STATES as usize / 2 {
            0
        } else {
            N_I_COORD_STATES / 2
        };

        let cube = CubieCube {
            orientation: Orientation::SOLVED,
            permutation: Permutation::from_coords(0, i_coord, i as u16),
        };

        for (j, val) in entry.iter_mut().enumerate() {
            *val = cube
                .apply_move(Move::<Phase3>::from_u8(j as u8))
                .permutation
                .o_coord();
        }
    });

    table
}
