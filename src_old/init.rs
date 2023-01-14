use crate::{
    cubiecube::{CubieCube, Orientation, Permutation},
    groups::A4Elem,
    node::{N_C3_COORD_STATES, N_IO_COORD_STATES, N_I_COORD_STATES, N_O_COORD_STATES},
    phase1::N_PHASE1_MOVES,
    phase2::N_PHASE2_MOVES,
    phase3::N_PHASE3_MOVES,
    puzzle::AxisEnum,
};

pub fn get_move_cubiecubes() -> [CubieCube; 92] {
    todo!()
}

pub fn get_move_axes() -> [AxisEnum; N_PHASE1_MOVES as usize] {
    todo!()
}

pub fn get_permutation_move_table() -> [Permutation; N_PHASE1_MOVES as usize] {
    let mut result = [Permutation::solved(); N_PHASE1_MOVES as usize];
    
    for i in 0..N_PHASE1_MOVES as usize {
        result[i] = CubieCube::solved().apply_move(i).permutation;
    }

    result
}

pub fn get_c3_move_table() -> Box<[[u32; N_PHASE2_MOVES as usize]; N_C3_COORD_STATES as usize]>
{   
    let mut result = Box::<[[u32; N_PHASE2_MOVES as usize]; N_C3_COORD_STATES as usize]>::new()

    for i in 0..N_C3_COORD_STATES as usize {
        for j in 0..N_PHASE2_MOVES as usize {
            (CubieCube { orientation: Orientation::from_c3_coord(i), permutation: Permutation::solved()}).apply_move(j).orientation.to_c3().c3_coord();
        }
    }
    
}

pub fn get_a4_move_table() -> [Orientation<A4Elem>; N_PHASE1_MOVES as usize] {
    let mut result = [Orientation::<A4Elem>::solved(); N_PHASE1_MOVES as usize];
    
    for i in 0..N_PHASE1_MOVES as usize {
        result[i] = CubieCube::solved().apply_move(i).orientation;
    }

    result
}

pub fn get_io_move_table() -> [[u16; N_PHASE2_MOVES as usize]; N_IO_COORD_STATES as usize] {
    todo!()
}

pub fn get_i_move_table() -> [[u16; N_PHASE3_MOVES as usize]; N_I_COORD_STATES as usize] {
    todo!()
}

pub fn get_o_move_table() -> [[u16; N_PHASE3_MOVES as usize]; N_O_COORD_STATES as usize] {
    todo!()
}
