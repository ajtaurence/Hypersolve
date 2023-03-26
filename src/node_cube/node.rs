use crate::{
    cubiecube::{
        cubiecube::{
            CubieCube, Orientation, Permutation, A4_MOVE_TABLE, HYPERSOLVE_TWISTS, PERM_MOVE_TABLE,
        },
        groups::K4,
    },
    math,
    piece_cube::pieces::Axis,
};

use itertools::Itertools;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

pub const N_K4_COORD_STATES: u32 = 4_u32.pow(15);
pub const N_C3_COORD_STATES: u32 = 3_u32.pow(14);
pub const N_IO_COORD_STATES: u16 = math::n_choose_k(15, 7);
pub const N_I_COORD_STATES: u16 = math::factorial(8) as u16;
pub const N_O_COORD_STATES: u16 = math::factorial(7) as u16;

pub const N_PHASE1_MOVES: u8 = 92;
pub const N_PHASE2_MOVES: u8 = 44;
pub const N_PHASE3_MOVES: u8 = 12;

const_data!(pub IO_MOVE_TABLE: [[u16; N_PHASE2_MOVES as usize]; N_IO_COORD_STATES as usize] = gen_io_move_table());
const_data!(pub I_MOVE_TABLE: [[u16; N_PHASE3_MOVES as usize]; N_I_COORD_STATES as usize] = gen_i_move_table());
const_data!(pub O_MOVE_TABLE: [[u16; N_PHASE3_MOVES as usize]; N_O_COORD_STATES as usize] = gen_o_move_table());
const_data!(pub C3_MOVE_TABLE: [[u32; N_PHASE2_MOVES as usize]; N_C3_COORD_STATES as usize] = gen_c3_move_table());
const_data!(pub MOVE_AXIS: [Axis; N_PHASE1_MOVES as usize] = gen_move_axis_table());

#[allow(unused)]
fn gen_move_axis_table() -> Box<[Axis; N_PHASE1_MOVES as usize]> {
    HYPERSOLVE_TWISTS
        .iter()
        .map(|&twist| twist.axis())
        .collect_vec()
        .try_into()
        .unwrap()
}

#[allow(unused)]
fn gen_c3_move_table() -> Box<[[u32; N_PHASE2_MOVES as usize]; N_C3_COORD_STATES as usize]> {
    let mut table = vec![[0_u32; N_PHASE2_MOVES as usize]; N_C3_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        let cube = CubieCube {
            orientation: Orientation::from_c3_coord(i as u32).into(),
            permutation: Permutation::solved(),
        };

        for j in 0..(N_PHASE2_MOVES as usize) {
            entry[j] = cube.apply_move(j).orientation.c3_coord();
        }
    });

    table.try_into().unwrap()
}

#[allow(unused)]
fn gen_io_move_table() -> Box<[[u16; N_PHASE2_MOVES as usize]; N_IO_COORD_STATES as usize]> {
    let mut table = vec![[0_u16; N_PHASE2_MOVES as usize]; N_IO_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        let cube = CubieCube {
            orientation: Orientation::solved(),
            permutation: Permutation::from_coords(i as u16, 0, 0),
        };

        for j in 0..(N_PHASE2_MOVES as usize) {
            entry[j] = cube.apply_move(j).permutation.io_coord();
        }
    });

    table.try_into().unwrap()
}

#[allow(unused)]
fn gen_i_move_table() -> Box<[[u16; N_PHASE3_MOVES as usize]; N_I_COORD_STATES as usize]> {
    let mut table = vec![[0_u16; N_PHASE3_MOVES as usize]; N_I_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        let cube = CubieCube {
            orientation: Orientation::solved(),
            permutation: Permutation::from_coords(0, i as u16, 0),
        };

        for j in 0..(N_PHASE3_MOVES as usize) {
            entry[j] = cube.apply_move(j).permutation.i_coord();
        }
    });

    table.try_into().unwrap()
}

#[allow(unused)]
fn gen_o_move_table() -> Box<[[u16; N_PHASE3_MOVES as usize]; N_O_COORD_STATES as usize]> {
    let mut table = vec![[0_u16; N_PHASE3_MOVES as usize]; N_O_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        let cube = CubieCube {
            orientation: Orientation::solved(),
            permutation: Permutation::from_coords(0, 0, i as u16),
        };

        for j in 0..(N_PHASE3_MOVES as usize) {
            entry[j] = cube.apply_move(j).permutation.o_coord();
        }
    });

    table.try_into().unwrap()
}

pub trait Node: Default + PartialEq + Copy + From<CubieCube> {
    const N_STATES: usize;

    /// Returns the index of the node
    fn get_index(&self) -> usize;

    /// Returns a node from an index
    fn from_index(index: usize, last_axis: Option<Axis>) -> Self;

    /// Returns a vector of the nodes connected to this node
    fn connected(&self) -> Vec<Self>
    where
        Self: Sized;

    /// Returns whether the node is the goal node
    fn is_goal(&self) -> bool {
        *self == Self::default()
    }

    /// Returns the goal node
    fn goal() -> Self {
        Self::default()
    }
}

#[derive(Default, Clone, Copy)]
pub struct Phase1Node {
    orientation: Orientation<K4>,
    last_axis: Option<Axis>,
}

impl PartialEq for Phase1Node {
    fn eq(&self, other: &Self) -> bool {
        self.orientation == other.orientation
    }
}

impl Node for Phase1Node {
    const N_STATES: usize = N_K4_COORD_STATES as usize;

    fn get_index(&self) -> usize {
        self.orientation.k4_coord() as usize
    }

    fn from_index(index: usize, last_axis: Option<Axis>) -> Self {
        Phase1Node {
            orientation: Orientation::<K4>::from_k4_coord(index as u32),
            last_axis,
        }
    }

    // TODO: Improve move filtering
    fn connected(&self) -> Vec<Self> {
        PERM_MOVE_TABLE
            .iter()
            .zip(A4_MOVE_TABLE.iter())
            .zip(MOVE_AXIS.into_iter())
            .filter(|(_, axis)| Some(*axis) != self.last_axis)
            .map(|((permutation, orientation), axis)| Phase1Node {
                orientation: self
                    .orientation
                    .apply_orientation(orientation.permute(*permutation)),
                last_axis: Some(axis),
            })
            .collect()
    }
}

impl From<CubieCube> for Phase1Node {
    fn from(value: CubieCube) -> Self {
        Phase1Node {
            orientation: value.orientation.into(),
            last_axis: None,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Phase2Node {
    c3_coord: u32,
    io_coord: u16,
    last_axis: Option<Axis>,
}

impl PartialEq for Phase2Node {
    fn eq(&self, other: &Self) -> bool {
        self.c3_coord == other.c3_coord && self.io_coord == other.io_coord
    }
}

impl Node for Phase2Node {
    const N_STATES: usize = N_C3_COORD_STATES as usize * N_IO_COORD_STATES as usize;

    fn get_index(&self) -> usize {
        (self.io_coord as usize) * (N_C3_COORD_STATES as usize) + (self.c3_coord as usize)
    }

    fn from_index(index: usize, _last_axis: Option<Axis>) -> Self {
        Phase2Node {
            c3_coord: (index % N_C3_COORD_STATES as usize) as u32,
            io_coord: (index / N_C3_COORD_STATES as usize) as u16,
            last_axis: None,
        }
    }

    // TODO: Improve move filtering
    fn connected(&self) -> Vec<Self> {
        C3_MOVE_TABLE[self.c3_coord as usize]
            .into_iter()
            .zip(IO_MOVE_TABLE[self.io_coord as usize].into_iter())
            .zip(MOVE_AXIS.into_iter())
            .filter(|(_, axis)| Some(*axis) != self.last_axis)
            .map(|((c3_coord, io_coord), axis)| Phase2Node {
                c3_coord,
                io_coord,
                last_axis: Some(axis),
            })
            .collect()
    }
}

impl From<CubieCube> for Phase2Node {
    fn from(value: CubieCube) -> Self {
        Phase2Node {
            c3_coord: value.orientation.c3_coord(),
            io_coord: value.permutation.io_coord(),
            last_axis: None,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Phase3Node {
    i_coord: u16,
    o_coord: u16,
    last_axis: Option<Axis>,
}

impl PartialEq for Phase3Node {
    fn eq(&self, other: &Self) -> bool {
        self.i_coord == other.i_coord && self.o_coord == other.o_coord
    }
}

impl Node for Phase3Node {
    const N_STATES: usize = N_I_COORD_STATES as usize * N_O_COORD_STATES as usize;

    fn get_index(&self) -> usize {
        self.o_coord as usize * (N_I_COORD_STATES / 2) as usize + (self.i_coord % 2) as usize
    }

    fn from_index(index: usize, last_axis: Option<Axis>) -> Self {
        let o_coord = (index / (N_I_COORD_STATES / 2) as usize) as u16;

        let i_coord = if o_coord >= (N_O_COORD_STATES / 2) as u16 {
            (index % (N_I_COORD_STATES / 2) as usize) as u16
        } else {
            (index % (N_I_COORD_STATES / 2) as usize) as u16 + (N_I_COORD_STATES / 2)
        };

        Phase3Node {
            i_coord,
            o_coord,
            last_axis,
        }
    }

    // TODO: Improve move filtering
    fn connected(&self) -> Vec<Self> {
        I_MOVE_TABLE[self.i_coord as usize]
            .into_iter()
            .zip(O_MOVE_TABLE[self.o_coord as usize].into_iter())
            .zip(MOVE_AXIS.into_iter())
            .filter(|(_, axis)| Some(*axis) != self.last_axis)
            .map(|((i_coord, o_coord), axis)| Phase3Node {
                i_coord,
                o_coord,
                last_axis: Some(axis),
            })
            .collect()
    }
}

impl From<CubieCube> for Phase3Node {
    fn from(value: CubieCube) -> Self {
        Phase3Node {
            i_coord: value.permutation.i_coord(),
            o_coord: value.permutation.o_coord(),
            last_axis: None,
        }
    }
}
