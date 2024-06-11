use derivative::Derivative;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use rkyv::Archive;
use std::hash::Hash;

use crate::common::groups::{Identity, K4};
use crate::common::math;
use crate::puzzle::Axis;

use super::*;

pub const N_K4_COORD_STATES: u32 = 4_u32.pow(15);
pub const N_C3_COORD_STATES: u32 = 3_u32.pow(14);
pub const N_IO_COORD_STATES: u16 = math::n_choose_k(15, 7);
pub const N_I_COORD_STATES: u16 = math::factorial(8) as u16;
pub const N_O_COORD_STATES: u16 = math::factorial(7) as u16;

const_data!(pub IO_MOVE_TABLE: [[u16; Phase2::N_MOVES ]; N_IO_COORD_STATES as usize] = gen_io_move_table());
const_data!(pub I_MOVE_TABLE: [[u16;  Phase3::N_MOVES]; N_I_COORD_STATES as usize] = gen_i_move_table());
const_data!(pub O_MOVE_TABLE: [[u16;  Phase3::N_MOVES]; N_O_COORD_STATES as usize] = gen_o_move_table());
const_data!(pub MOVE_AXIS: [Axis;  Phase1::N_MOVES] = gen_move_axis_table());

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_io_move_table() {
    let _ = &*IO_MOVE_TABLE;
}

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_i_move_table() {
    let _ = &*I_MOVE_TABLE;
}

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_o_move_table() {
    let _ = &*O_MOVE_TABLE;
}

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_move_axis_table() {
    let _ = &*MOVE_AXIS;
}

#[cfg(feature = "gen-const-data")]
fn gen_move_axis_table() -> Box<[Axis; Phase1::N_MOVES]> {
    use itertools::Itertools;
    HYPERSOLVE_TWISTS
        .iter()
        .map(|&twist| twist.axis())
        .collect_vec()
        .try_into()
        .unwrap()
}

fn gen_c3_move_table() -> Box<[[u32; Phase2::N_MOVES]; N_C3_COORD_STATES as usize]> {
    let mut table = vec![[0_u32; Phase2::N_MOVES]; N_C3_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        let cube = CubieCube {
            orientation: Orientation::from_c3_coord(i as u32).into(),
            permutation: Permutation::solved(),
        };

        for (j, val) in entry.iter_mut().enumerate().take(Phase2::N_MOVES) {
            *val = cube.apply_move(Move(j as u8)).orientation.c3_coord();
        }
    });

    table.try_into().unwrap()
}

#[cfg(feature = "gen-const-data")]
fn gen_io_move_table() -> Box<[[u16; Phase2::N_MOVES]; N_IO_COORD_STATES as usize]> {
    let mut table = vec![[0_u16; Phase2::N_MOVES]; N_IO_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        let cube = CubieCube {
            orientation: Orientation::solved(),
            permutation: Permutation::from_coords(i as u16, 0, 0),
        };

        for (j, val) in entry.iter_mut().enumerate() {
            *val = cube.apply_move(Move(j as u8)).permutation.io_coord();
        }
    });

    table.try_into().unwrap()
}

#[cfg(feature = "gen-const-data")]
fn gen_i_move_table() -> Box<[[u16; Phase3::N_MOVES]; N_I_COORD_STATES as usize]> {
    let mut table = vec![[0_u16; Phase3::N_MOVES]; N_I_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        // Ensure the total permutation parity of the cube remains valid
        let o_coord = if i < N_I_COORD_STATES as usize / 2 {
            0
        } else {
            N_O_COORD_STATES / 2
        };

        let cube = CubieCube {
            orientation: Orientation::solved(),
            permutation: Permutation::from_coords(0, i as u16, o_coord),
        };

        for (j, val) in entry.iter_mut().enumerate() {
            *val = cube.apply_move(Move(j as u8)).permutation.i_coord();
        }
    });

    table.try_into().unwrap()
}

#[cfg(feature = "gen-const-data")]
fn gen_o_move_table() -> Box<[[u16; Phase3::N_MOVES]; N_O_COORD_STATES as usize]> {
    let mut table = vec![[0_u16; Phase3::N_MOVES]; N_O_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        // Ensure the total permutation parity of the cube remains valid
        let i_coord = if i < N_O_COORD_STATES as usize / 2 {
            0
        } else {
            N_I_COORD_STATES / 2
        };

        let cube = CubieCube {
            orientation: Orientation::solved(),
            permutation: Permutation::from_coords(0, i_coord, i as u16),
        };

        for (j, val) in entry.iter_mut().enumerate() {
            *val = cube.apply_move(Move(j as u8)).permutation.o_coord();
        }
    });

    table.try_into().unwrap()
}

/// A trait for highly optimized computation of how certain aspects of the cube
/// are affected by twists.
pub(crate) trait Node: Identity + PartialEq + Copy + From<CubieCube> {
    const N_STATES: usize;

    type Phase: Phase;
    type Index: Into<u64> + Archive<Archived = Self::Index> + Hash + std::cmp::Eq;

    /// Returns the index of the node
    fn get_index(&self) -> Self::Index;

    /// Returns a node from an index
    fn from_index(index: u64, last_move: Option<Move>) -> Self;

    /// Returns the last move applied to the node
    fn last_move(&self) -> Option<Move>;

    /// Returns the axis of the last move applied to the node
    fn last_axis(&self) -> Option<Axis> {
        self.last_move().map(|m| m.axis())
    }

    /// Gets the lower bound on the number of moves requied to reach the goal node from this node
    fn get_depth_bound(&self) -> u8;

    /// Applies the given move to the node
    fn apply_move(self, move_index: Move) -> Self;

    /// Returns a vector of the nodes connected to this node using the given connected node iterator
    fn connected<I: ConnectedNodeIterator<Self>>(&self) -> I {
        I::new(*self)
    }

    /// Returns whether the node is the goal node
    fn is_goal(&self) -> bool {
        *self == Self::IDENTITY
    }

    /// Returns the goal node
    fn goal() -> Self {
        Self::IDENTITY
    }
}

/// A node representing a cube state in phase 1
#[derive(Derivative)]
#[derivative(PartialEq, Eq)]
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Phase1Node {
    orientation: Orientation<K4>,

    #[derivative(PartialEq = "ignore")]
    last_move: Option<Move>,
}

impl Phase1Node {
    #[allow(unused)]
    pub fn apply_move(self, i: usize) -> Self {
        Phase1Node {
            orientation: self
                .orientation
                .permute(PERM_MOVE_TABLE[i])
                .apply_orientation(A4_MOVE_TABLE[i]),
            last_move: Some(Move(i as u8)),
        }
    }
}

impl Identity for Phase1Node {
    const IDENTITY: Self = Phase1Node {
        orientation: Orientation::<K4>::IDENTITY,
        last_move: None,
    };
}

impl Node for Phase1Node {
    const N_STATES: usize = N_K4_COORD_STATES as usize;
    type Phase = Phase1;
    type Index = u32;

    fn get_index(&self) -> u32 {
        self.orientation.k4_coord()
    }

    fn from_index(index: u64, last_move: Option<Move>) -> Self {
        Phase1Node {
            orientation: Orientation::<K4>::from_k4_coord(index as u32),
            last_move,
        }
    }

    fn last_move(&self) -> Option<Move> {
        self.last_move
    }

    fn apply_move(self, move_index: Move) -> Self {
        unsafe { assert_unchecked!(move_index.as_usize() < Phase1::N_MOVES) };

        let perm = PERM_MOVE_TABLE[move_index.as_usize()];
        let orien = A4_MOVE_TABLE[move_index.as_usize()];
        Self {
            orientation: self.orientation.permute(perm).apply_orientation(orien),
            last_move: Some(move_index),
        }
    }

    fn get_depth_bound(&self) -> u8 {
        load_or_generate_data!(static PHASE1_PRUNING_TABLE: <Phase1 as Phase>::PruningTable = <Phase1 as Phase>::PruningTable::generate(Phase1::PRUNING_DEPTH), "phase1.prun");

        PHASE1_PRUNING_TABLE.get_depth_bound(*self)
    }
}

impl From<CubieCube> for Phase1Node {
    fn from(value: CubieCube) -> Self {
        Phase1Node {
            orientation: value.orientation.into(),
            last_move: None,
        }
    }
}

/// A node representing a cube state in phase 2
#[derive(Derivative)]
#[derivative(PartialEq, Eq)]
#[derive(Default, Clone, Copy)]
pub(crate) struct Phase2Node {
    pub c3_coord: u32,
    pub io_coord: u16,

    #[derivative(PartialEq = "ignore")]
    pub last_move: Option<Move>,
}

impl Identity for Phase2Node {
    const IDENTITY: Self = Phase2Node {
        c3_coord: 0,
        io_coord: 0,
        last_move: None,
    };
}

impl Node for Phase2Node {
    const N_STATES: usize = N_C3_COORD_STATES as usize * N_IO_COORD_STATES as usize;
    type Phase = Phase2;
    type Index = u64;

    fn get_index(&self) -> u64 {
        (self.io_coord as u64) * (N_C3_COORD_STATES as u64) + (self.c3_coord as u64)
    }

    fn from_index(index: u64, last_move: Option<Move>) -> Self {
        Phase2Node {
            c3_coord: (index % N_C3_COORD_STATES as u64) as u32,
            io_coord: (index / N_C3_COORD_STATES as u64) as u16,
            last_move,
        }
    }

    fn last_move(&self) -> Option<Move> {
        self.last_move
    }

    fn apply_move(self, move_index: Move) -> Self {
        load_or_generate_data!(static C3_MOVE_TABLE: Box<[[u32; Phase2::N_MOVES]; N_C3_COORD_STATES as usize]> = gen_c3_move_table(), "c3.move");

        unsafe {
            assert_unchecked!(move_index.as_usize() < Phase2::N_MOVES);
            assert_unchecked!(self.c3_coord < N_C3_COORD_STATES);
            assert_unchecked!(self.io_coord < N_IO_COORD_STATES);
        }

        let c3_coord = C3_MOVE_TABLE[self.c3_coord as usize][move_index.as_usize()];
        let io_coord = IO_MOVE_TABLE[self.io_coord as usize][move_index.as_usize()];
        Self {
            c3_coord,
            io_coord,
            last_move: Some(move_index),
        }
    }

    fn get_depth_bound(&self) -> u8 {
        load_or_generate_data!(static PHASE2_PRUNING_TABLE: <Phase2 as Phase>::PruningTable = <Phase2 as Phase>::PruningTable::generate(Phase2::PRUNING_DEPTH), "phase2.prun");

        PHASE2_PRUNING_TABLE.get_depth_bound(*self)
    }
}

impl From<CubieCube> for Phase2Node {
    fn from(value: CubieCube) -> Self {
        Phase2Node {
            c3_coord: value.orientation.c3_coord(),
            io_coord: value.permutation.io_coord(),
            last_move: None,
        }
    }
}

/// A node representing a cube state in phase 3
#[derive(Derivative)]
#[derivative(PartialEq, Eq)]
#[derive(Default, Clone, Copy)]
pub(crate) struct Phase3Node {
    pub i_coord: u16,
    pub o_coord: u16,

    #[derivative(PartialEq = "ignore")]
    pub last_move: Option<Move>,
}

impl Identity for Phase3Node {
    const IDENTITY: Self = Phase3Node {
        i_coord: 0,
        o_coord: 0,
        last_move: None,
    };
}

impl Node for Phase3Node {
    const N_STATES: usize = N_I_COORD_STATES as usize * N_O_COORD_STATES as usize / 2;
    type Phase = Phase3;
    type Index = u32;

    fn get_index(&self) -> u32 {
        self.o_coord as u32 * (N_I_COORD_STATES / 2) as u32
            + (self.i_coord as u32 % (N_I_COORD_STATES / 2) as u32)
    }

    fn from_index(index: u64, last_move: Option<Move>) -> Self {
        let o_coord = (index / (N_I_COORD_STATES / 2) as u64) as u16;
        let mut i_coord = (index % (N_I_COORD_STATES / 2) as u64) as u16;

        // if O piece permutation is odd then the I piece permutation must also be odd
        if o_coord >= (N_O_COORD_STATES / 2) {
            i_coord += N_I_COORD_STATES / 2;
        };

        Phase3Node {
            i_coord,
            o_coord,
            last_move,
        }
    }

    fn last_move(&self) -> Option<Move> {
        self.last_move
    }

    fn apply_move(self, move_index: Move) -> Self {
        unsafe {
            assert_unchecked!(move_index.as_usize() < Phase3::N_MOVES);
            assert_unchecked!(self.i_coord < N_I_COORD_STATES);
            assert_unchecked!(self.o_coord < N_O_COORD_STATES);
        }

        let i_coord = I_MOVE_TABLE[self.i_coord as usize][move_index.as_usize()];
        let o_coord = O_MOVE_TABLE[self.o_coord as usize][move_index.as_usize()];
        Self {
            i_coord,
            o_coord,
            last_move: Some(move_index),
        }
    }

    fn get_depth_bound(&self) -> u8 {
        load_or_generate_data!(static PHASE3_PRUNING_TABLE: <Phase3 as Phase>::PruningTable = <Phase3 as Phase>::PruningTable::generate(Phase3::PRUNING_DEPTH), "phase3.prun");

        PHASE3_PRUNING_TABLE.get_depth_bound(*self)
    }
}

impl From<CubieCube> for Phase3Node {
    fn from(value: CubieCube) -> Self {
        Phase3Node {
            i_coord: value.permutation.i_coord(),
            o_coord: value.permutation.o_coord(),
            last_move: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orientation_to_from_c3_coord() {
        use crate::common::groups::A4;
        for i in (0..N_C3_COORD_STATES).step_by(10) {
            let orientation: Orientation<A4> = Orientation::from_c3_coord(i).into();
            assert_eq!(orientation.c3_coord(), i)
        }
    }

    #[test]
    fn test_permutation_to_from_io_coord() {
        for i in 0..N_IO_COORD_STATES {
            let permutation = Permutation::from_coords(i, 0, 0);
            assert_eq!(permutation.io_coord(), i)
        }
    }

    #[test]
    fn test_permutation_to_from_i_coord() {
        for i in 0..N_I_COORD_STATES / 2 {
            let permutation = Permutation::from_coords(0, i, 0);
            assert_eq!(permutation.i_coord(), i)
        }
    }

    #[test]
    fn test_permutation_to_from_o_coord() {
        for i in 0..N_O_COORD_STATES {
            let permutation = Permutation::from_coords(0, 0, i);
            assert_eq!(permutation.o_coord(), i)
        }
    }
}
