use crate::common::Axis;
use crate::cubiecube::{CubieCube, Orientation, Permutation, A4_MOVE_TABLE, PERM_MOVE_TABLE};
use crate::groups::K4;
use crate::math;

use lazy_static::lazy_static;

pub const N_K4_COORD_STATES: u32 = 4_u32.pow(15);
pub const N_C3_COORD_STATES: u32 = 3_u32.pow(14);
pub const N_IO_COORD_STATES: u16 = math::n_choose_k(15, 7);
pub const N_I_COORD_STATES: u16 = math::factorial(8) as u16;
pub const N_O_COORD_STATES: u16 = math::factorial(7) as u16;

pub const N_PHASE1_MOVES: u8 = 92;
pub const N_PHASE2_MOVES: u8 = 44;
pub const N_PHASE3_MOVES: u8 = 12;

// Needs to be defined differently because bytemuck does not allow nested arrays longer than 32 entries
const_lookup_table!(
    IO_MOVE_TABLE: &'static [u16; N_PHASE2_MOVES as usize * N_IO_COORD_STATES as usize] =
        ("io.move", gen_io_move_table)
);
const_lookup_table!(
    I_MOVE_TABLE: &'static [[u16; N_PHASE3_MOVES as usize]; N_I_COORD_STATES as usize] =
        ("i.move", gen_i_move_table)
);
const_lookup_table!(
    O_MOVE_TABLE: &'static [[u16; N_PHASE3_MOVES as usize]; N_O_COORD_STATES as usize] =
        ("o.move", gen_o_move_table)
);
const_lookup_table!(
    MOVE_AXIS: &'static [Axis; N_PHASE3_MOVES as usize] = ("move_axis.data", gen_move_axis_table)
);

// TODO: try to load from file or generate if needed
lazy_static! {
    pub static ref C3_MOVE_TABLE: [u32; N_PHASE2_MOVES as usize * N_C3_COORD_STATES as usize] =
        gen_c3_move_table();
}

fn gen_move_axis_table() -> &'static [Axis; N_PHASE3_MOVES as usize] {
    todo!()
}

fn gen_c3_move_table() -> [u32; N_PHASE2_MOVES as usize * N_C3_COORD_STATES as usize] {
    static table: [u32; N_PHASE2_MOVES as usize * N_C3_COORD_STATES as usize] =
        [0_u32; N_PHASE2_MOVES as usize * N_C3_COORD_STATES as usize];

    for i in 0..N_C3_COORD_STATES {
        let cube = CubieCube {
            orientation: Orientation::from_c3_coord(i).into(),
            permutation: Permutation::solved(),
        };

        for j in 0..(N_PHASE2_MOVES as usize) {
            table[i as usize * N_PHASE2_MOVES as usize + j] =
                cube.apply_move(j).orientation.c3_coord();
        }
    }

    table
}

fn gen_io_move_table() -> &'static [u16; N_PHASE2_MOVES as usize * N_IO_COORD_STATES as usize] {
    static table: [u16; N_PHASE2_MOVES as usize * N_IO_COORD_STATES as usize] =
        [0_u16; N_PHASE2_MOVES as usize * N_IO_COORD_STATES as usize];

    for i in 0..N_IO_COORD_STATES {
        let cube = CubieCube {
            orientation: Orientation::solved(),
            permutation: Permutation::from_coords(i, 0, 0),
        };

        for j in 0..(N_PHASE2_MOVES as usize) {
            table[i as usize * N_PHASE2_MOVES as usize + j] =
                cube.apply_move(j).permutation.o_coord();
        }
    }

    &table
}

fn gen_i_move_table() -> &'static [[u16; N_PHASE3_MOVES as usize]; N_I_COORD_STATES as usize] {
    static table: [[u16; N_PHASE3_MOVES as usize]; N_I_COORD_STATES as usize] =
        [[0_u16; N_PHASE3_MOVES as usize]; N_I_COORD_STATES as usize];

    for i in 0..N_I_COORD_STATES {
        let cube = CubieCube {
            orientation: Orientation::solved(),
            permutation: Permutation::from_coords(0, i, 0),
        };

        for j in 0..(N_PHASE3_MOVES as usize) {
            table[i as usize][j] = cube.apply_move(j).permutation.i_coord();
        }
    }

    &table
}

fn gen_o_move_table() -> &'static [[u16; N_PHASE3_MOVES as usize]; N_O_COORD_STATES as usize] {
    static table: [[u16; N_PHASE3_MOVES as usize]; N_O_COORD_STATES as usize] =
        [[0_u16; N_PHASE3_MOVES as usize]; N_O_COORD_STATES as usize];

    for i in 0..N_I_COORD_STATES {
        let cube = CubieCube {
            orientation: Orientation::solved(),
            permutation: Permutation::from_coords(0, 0, i),
        };

        for j in 0..(N_PHASE3_MOVES as usize) {
            table[i as usize][j] = cube.apply_move(j).permutation.o_coord();
        }
    }

    &table
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
            .filter(|(_, &axis)| Some(axis) != self.last_axis)
            .map(|((permutation, orientation), axis)| Phase1Node {
                orientation: self
                    .orientation
                    .apply_orientation(orientation.permute(*permutation)),
                last_axis: Some(*axis),
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
        C3_MOVE_TABLE[self.c3_coord as usize * N_PHASE2_MOVES as usize
            ..(self.c3_coord as usize + 1) * N_PHASE2_MOVES as usize]
            .into_iter()
            .zip(
                IO_MOVE_TABLE[self.io_coord as usize * N_PHASE2_MOVES as usize
                    ..(self.io_coord as usize + 1) * N_PHASE2_MOVES as usize]
                    .into_iter(),
            )
            .zip(MOVE_AXIS.into_iter())
            .filter(|(_, &axis)| Some(axis) != self.last_axis)
            .map(|((&c3_coord, &io_coord), axis)| Phase2Node {
                c3_coord,
                io_coord,
                last_axis: Some(*axis),
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
            .filter(|(_, &axis)| Some(axis) != self.last_axis)
            .map(|((i_coord, o_coord), axis)| Phase3Node {
                i_coord,
                o_coord,
                last_axis: Some(*axis),
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