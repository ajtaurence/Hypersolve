use crate::cubiecube::{Orientation, Permutation};
use crate::groups::{A4Elem, K4Elem};
use crate::init::{
    get_a4_move_table, get_c3_move_table, get_i_move_table, get_io_move_table, get_o_move_table,
    get_permutation_move_table,
};
use crate::phase1::N_PHASE1_MOVES;
use crate::phase2::N_PHASE2_MOVES;
use crate::phase3::N_PHASE3_MOVES;
use crate::puzzle::AxisEnum;
use crate::utils::MOVE_AXIS;

pub const N_C3_COORD_STATES: u32 = 4782969;
pub const N_IO_COORD_STATES: u16 = 6435;
pub const N_I_COORD_STATES: u16 = 40320;
pub const N_O_COORD_STATES: u16 = 5040;

//TODO: Figure out how to generate and include data at compile time
pub static PERM_MOVE_TABLE: [Permutation; N_PHASE1_MOVES as usize] = get_permutation_move_table();
pub static  A4_MOVE_TABLE: [Orientation<A4Elem>; N_PHASE1_MOVES as usize] = get_a4_move_table();
lazy_static! {
    pub static ref C3_MOVE_TABLE: Box<[[u32; N_PHASE2_MOVES as usize]; N_C3_COORD_STATES as usize]> =
        get_c3_move_table();
}
pub static IO_MOVE_TABLE: [[u16; N_PHASE2_MOVES as usize]; N_IO_COORD_STATES as usize] =
    get_io_move_table();
pub static I_MOVE_TABLE: [[u16; N_PHASE3_MOVES as usize]; N_I_COORD_STATES as usize] =
    get_i_move_table();
pub static O_MOVE_TABLE: [[u16; N_PHASE3_MOVES as usize]; N_O_COORD_STATES as usize] =
    get_o_move_table();

pub trait Node {
    // Returns the index of the node
    fn get_index(&self) -> usize;

    // Returns a node from an index
    fn from_index(index: usize, last_axis: Option<AxisEnum>) -> Self;

    // Returns a vector of the nodes connected to this node
    fn connected(&self) -> Vec<Self>
    where
        Self: Sized;

    // Returns whether the node is the goal node
    fn is_goal(&self) -> bool;

    // Returns the goal node
    fn goal() -> Self;
}

#[derive(PartialEq)]
pub struct Phase1Node {
    orientation: Orientation<K4Elem>,
    last_axis: Option<AxisEnum>,
}

impl Phase1Node {
    pub fn new(orientation: Orientation<K4Elem>, last_axis: Option<AxisEnum>) -> Phase1Node {
        Phase1Node {
            orientation,
            last_axis,
        }
    }
}

impl Node for Phase1Node {
    #[inline]
    fn get_index(&self) -> usize {
        self.orientation.to_int() as usize
    }

    #[inline]
    fn from_index(index: usize, last_axis: Option<AxisEnum>) -> Self {
        Phase1Node {
            orientation: Orientation::<K4Elem>::from_int(index as u32),
            last_axis,
        }
    }

    // TODO: Improve move filtering
    fn connected(&self) -> Vec<Self> {
        PERM_MOVE_TABLE
            .iter()
            .zip(A4_MOVE_TABLE.iter())
            .zip(MOVE_AXIS.into_iter())
            .filter(|((_), axis)| Some(*axis) != self.last_axis)
            .map(|((permutation, orientation), axis)| Phase1Node {
                orientation: self
                    .orientation
                    .apply_orientation(&orientation.permute(permutation)),
                last_axis: Some(axis),
            })
            .collect()
    }

    #[inline]
    fn is_goal(&self) -> bool {
        self.orientation.is_solved()
    }

    #[inline]
    fn goal() -> Self {
        Phase1Node {
            orientation: Orientation::<K4Elem>::solved(),
            last_axis: None,
        }
    }
}

pub struct Phase2Node {
    c3_coord: u32,
    io_coord: u16,
    last_axis: Option<AxisEnum>,
}

impl PartialEq for Phase2Node {
    fn eq(&self, other: &Self) -> bool {
        self.c3_coord == other.c3_coord && self.io_coord == other.io_coord
    }
}

impl Node for Phase2Node {
    fn get_index(&self) -> usize {
        (self.io_coord as usize) * (N_C3_COORD_STATES as usize) + (self.c3_coord as usize)
    }

    fn from_index(index: usize, _last_axis: Option<AxisEnum>) -> Self {
        Phase2Node {
            c3_coord: (index % N_C3_COORD_STATES as usize) as u32,
            io_coord: (index / N_C3_COORD_STATES as usize) as u16,
            last_axis: None,
        }
    }

    // TODO: Improve move filtering
    fn connected(&self) -> Vec<Self> {
        C3_MOVE_TABLE[self.get_index()]
            .into_iter()
            .zip(IO_MOVE_TABLE[self.get_index()].into_iter())
            .zip(MOVE_AXIS.into_iter())
            .filter(|((_), axis)| Some(*axis) != self.last_axis)
            .map(|((c3_coord, io_coord), axis)| Phase2Node {
                c3_coord,
                io_coord,
                last_axis: Some(axis),
            })
            .collect()
    }

    fn is_goal(&self) -> bool {
        self.c3_coord == 0 && self.io_coord == 0
    }

    fn goal() -> Self {
        Phase2Node {
            c3_coord: 0,
            io_coord: 0,
            last_axis: None,
        }
    }
}

pub struct Phase3Node {
    i_coord: u16,
    o_coord: u16,
    last_axis: Option<AxisEnum>,
}

impl PartialEq for Phase3Node {
    fn eq(&self, other: &Self) -> bool {
        self.i_coord == other.i_coord && self.o_coord == other.o_coord
    }
}

impl Node for Phase3Node {
    fn get_index(&self) -> usize {
        self.o_coord as usize * (N_I_COORD_STATES / 2) as usize + (self.i_coord % 2) as usize
    }

    fn from_index(index: usize, last_axis: Option<AxisEnum>) -> Self {
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
        I_MOVE_TABLE[self.get_index()]
            .into_iter()
            .zip(O_MOVE_TABLE[self.get_index()].into_iter())
            .zip(MOVE_AXIS.into_iter())
            .filter(|((_), axis)| Some(*axis) != self.last_axis)
            .map(|((i_coord, o_coord), axis)| Phase3Node {
                i_coord,
                o_coord,
                last_axis: Some(axis),
            })
            .collect()
    }

    fn is_goal(&self) -> bool {
        self.i_coord == 0 && self.o_coord == 0
    }

    fn goal() -> Self {
        Phase3Node {
            i_coord: 0,
            o_coord: 0,
            last_axis: None,
        }
    }
}
